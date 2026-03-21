use std::collections::HashSet;
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::db::Db;
use crate::quests;

pub const SIDEBAR_ITEMS: [&str; 3] = ["  Levels", "  Instructions", "  Exit"];

// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum QuestFocus {
    Input,
    Run,
    Submit,
    Back,
}

impl QuestFocus {
    pub fn next(&self) -> Self {
        match self {
            Self::Input | Self::Run => Self::Submit,
            Self::Submit => Self::Back,
            Self::Back => Self::Run,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            Self::Input | Self::Submit => Self::Run,
            Self::Run => Self::Back,
            Self::Back => Self::Submit,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail(String),
}

#[derive(Debug)]
pub struct QuestViewState {
    pub quest_id: usize,
    pub input: String,
    pub cursor: usize,
    pub output: String,
    pub test_result: Option<TestResult>,
    pub focus: QuestFocus,
}

impl QuestViewState {
    pub fn new(quest_id: usize) -> Self {
        Self {
            quest_id,
            input: String::new(),
            cursor: 0,
            output: String::new(),
            test_result: None,
            focus: QuestFocus::Input,
        }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    pub sidebar_index: usize,
    pub reset_focused: bool,
    pub reset_done: bool,
    pub quest_list_index: usize,
    pub content_focused: bool,
    pub quest_view: Option<QuestViewState>,
    pub db: Db,
    pub completed: HashSet<usize>,
    pub exit: bool,
}

impl App {
    pub fn new() -> rusqlite::Result<Self> {
        let db = Db::open()?;
        let completed = db.completed_quests();
        Ok(Self {
            sidebar_index: 0,
            reset_focused: false,
            reset_done: false,
            quest_list_index: 0,
            content_focused: false,
            quest_view: None,
            db,
            completed,
            exit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| crate::ui::draw(frame, self))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                self.handle_key(key);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.quest_view.is_some() {
            self.handle_quest_key(key);
        } else if self.content_focused {
            self.handle_list_key(key);
        } else {
            self.handle_overview_key(key);
        }
    }

    // -----------------------------------------------------------------------
    // Overview (sidebar) navigation
    // -----------------------------------------------------------------------

    fn handle_overview_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit = true,
            KeyCode::Up => {
                if self.reset_focused {
                    self.reset_focused = false;
                    self.reset_done = false;
                } else if self.sidebar_index > 0 {
                    self.set_sidebar(self.sidebar_index - 1);
                }
            }
            KeyCode::Down => {
                if !self.reset_focused {
                    if self.sidebar_index < SIDEBAR_ITEMS.len() - 1 {
                        self.set_sidebar(self.sidebar_index + 1);
                    } else {
                        self.reset_focused = true;
                        self.reset_done = false;
                    }
                }
            }
            KeyCode::Enter => {
                if self.reset_focused {
                    let _ = self.db.reset_all();
                    self.completed.clear();
                    self.quest_list_index = 0;
                    self.reset_done = true;
                } else {
                    match self.sidebar_index {
                        0 => self.content_focused = true,
                        2 => self.exit = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Quest list (grid) navigation
    // -----------------------------------------------------------------------

    fn handle_list_key(&mut self, key: KeyEvent) {
        let cols = self.quest_grid_cols();
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc => self.content_focused = false,
            KeyCode::Up => {
                if self.quest_list_index >= cols {
                    self.quest_list_index -= cols;
                }
            }
            KeyCode::Down => {
                let next = self.quest_list_index + cols;
                if next < 24 {
                    self.quest_list_index = next;
                } else if self.quest_list_index / cols < 23 / cols {
                    self.quest_list_index = 23;
                }
            }
            KeyCode::Left => {
                if self.quest_list_index > 0 {
                    self.quest_list_index -= 1;
                }
            }
            KeyCode::Right => {
                if self.quest_list_index < 23 {
                    self.quest_list_index += 1;
                }
            }
            KeyCode::Enter => {
                self.quest_view = Some(QuestViewState::new(self.quest_list_index + 1));
            }
            _ => {}
        }
    }

    /// Estimate grid columns from the current terminal width.
    pub fn quest_grid_cols(&self) -> usize {
        let term_w = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);
        let content_w = term_w.saturating_sub(22).saturating_sub(2); // sidebar + borders
        let box_w = 10u16;
        let gap = 1u16;
        ((content_w + gap) / (box_w + gap)).max(1) as usize
    }

    // -----------------------------------------------------------------------
    // Quest view key handling
    // -----------------------------------------------------------------------

    fn handle_quest_key(&mut self, key: KeyEvent) {
        // Resolve action without holding a borrow on self.quest_view
        let action = {
            let qv = self.quest_view.as_ref().unwrap();
            resolve_quest_action(qv, key)
        };
        self.apply_quest_action(action);
    }

    fn apply_quest_action(&mut self, action: QuestAction) {
        match action {
            QuestAction::None => {}
            QuestAction::Insert(c) => {
                let qv = self.quest_view.as_mut().unwrap();
                let pos = qv.cursor;
                qv.input.insert(pos, c);
                qv.cursor += 1;
            }
            QuestAction::Backspace => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor > 0 {
                    qv.cursor -= 1;
                    let pos = qv.cursor;
                    qv.input.remove(pos);
                }
            }
            QuestAction::CursorLeft => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor > 0 {
                    qv.cursor -= 1;
                }
            }
            QuestAction::CursorRight => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor < qv.input.len() {
                    qv.cursor += 1;
                }
            }
            QuestAction::Run => self.run_command(),
            QuestAction::Submit => self.submit_quest(),
            QuestAction::FocusInput => {
                self.quest_view.as_mut().unwrap().focus = QuestFocus::Input;
            }
            QuestAction::FocusNext => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.focus = qv.focus.next();
            }
            QuestAction::FocusPrev => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.focus = qv.focus.prev();
            }
            QuestAction::Back => {
                self.quest_view = None;
                self.content_focused = true;
            }
            QuestAction::Escape => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.test_result.is_some() {
                    qv.test_result = None;
                } else {
                    self.quest_view = None;
                    self.content_focused = true;
                }
            }
        }
    }

    fn run_command(&mut self) {
        let qv = self.quest_view.as_mut().unwrap();
        let cmd = qv.input.trim().to_string();
        if cmd.is_empty() {
            qv.output = "No command entered.".to_string();
            return;
        }
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output();
        qv.output = match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if stdout.is_empty() {
                    stderr
                } else if stderr.is_empty() {
                    stdout
                } else {
                    format!("{}\n---stderr---\n{}", stdout, stderr)
                }
            }
            Err(e) => format!("Failed to run command: {}", e),
        };
        qv.test_result = None;
        qv.focus = QuestFocus::Submit;
    }

    fn submit_quest(&mut self) {
        let quest_id = self.quest_view.as_ref().unwrap().quest_id;
        let output = self.quest_view.as_ref().unwrap().output.clone();
        if let Some(quest) = quests::get(quest_id) {
            let result = match (quest.verify)(&output) {
                quests::VerifyResult::Pass => {
                    self.completed.insert(quest_id);
                    let _ = self.db.mark_completed(quest_id);
                    TestResult::Pass
                }
                quests::VerifyResult::Fail(msg) => TestResult::Fail(msg),
            };
            self.quest_view.as_mut().unwrap().test_result = Some(result);
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    pub fn set_sidebar(&mut self, index: usize) {
        self.sidebar_index = index;
        self.content_focused = false;
        self.quest_view = None;
    }
}

// ---------------------------------------------------------------------------
// Quest action resolution (pure, no self borrow)
// ---------------------------------------------------------------------------

enum QuestAction {
    None,
    Insert(char),
    Backspace,
    CursorLeft,
    CursorRight,
    Run,
    Submit,
    FocusInput,
    FocusNext,
    FocusPrev,
    Back,
    Escape,
}

fn resolve_quest_action(qv: &QuestViewState, key: KeyEvent) -> QuestAction {
    match key.code {
        KeyCode::Esc => QuestAction::Escape,
        _ => match qv.focus {
            QuestFocus::Input => match key.code {
                KeyCode::Char(c) => QuestAction::Insert(c),
                KeyCode::Backspace => QuestAction::Backspace,
                KeyCode::Left => QuestAction::CursorLeft,
                KeyCode::Right => QuestAction::CursorRight,
                KeyCode::Enter => QuestAction::Run,
                KeyCode::Tab => QuestAction::FocusNext,
                _ => QuestAction::None,
            },
            QuestFocus::Run => match key.code {
                KeyCode::Enter => QuestAction::Run,
                KeyCode::Tab | KeyCode::Right => QuestAction::FocusNext,
                KeyCode::Left => QuestAction::FocusPrev,
                KeyCode::Char(_) => QuestAction::FocusInput,
                _ => QuestAction::None,
            },
            QuestFocus::Submit => match key.code {
                KeyCode::Enter => QuestAction::Submit,
                KeyCode::Tab | KeyCode::Right => QuestAction::FocusNext,
                KeyCode::Left => QuestAction::FocusPrev,
                KeyCode::Char(_) => QuestAction::FocusInput,
                _ => QuestAction::None,
            },
            QuestFocus::Back => match key.code {
                KeyCode::Enter => QuestAction::Back,
                KeyCode::Tab | KeyCode::Right => QuestAction::FocusNext,
                KeyCode::Left => QuestAction::FocusPrev,
                KeyCode::Char(_) => QuestAction::FocusInput,
                _ => QuestAction::None,
            },
        },
    }
}
