use std::cell::Cell;
use std::collections::HashSet;
use std::io;
use std::process::{Child, Stdio};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind};
use ratatui::DefaultTerminal;

use crate::db::{self, Db};
use crate::quests;

pub const SIDEBAR_ITEMS: [&str; 3] = ["  Levels", "  Instructions", "  Exit"];


// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum QuestFocus {
    Instructions,
    Terminal,
    Answer,
    Submit,
    Back,
}

impl QuestFocus {
    pub fn next(&self, has_answer: bool) -> Self {
        match self {
            Self::Instructions => Self::Terminal,
            Self::Terminal => {
                if has_answer {
                    Self::Answer
                } else {
                    Self::Submit
                }
            }
            Self::Answer => Self::Submit,
            Self::Submit => Self::Back,
            Self::Back => Self::Instructions,
        }
    }
    pub fn prev(&self, has_answer: bool) -> Self {
        match self {
            Self::Instructions => Self::Back,
            Self::Terminal => Self::Instructions,
            Self::Answer => Self::Terminal,
            Self::Submit => {
                if has_answer {
                    Self::Answer
                } else {
                    Self::Terminal
                }
            }
            Self::Back => Self::Submit,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail(String),
}

/// One entry in the terminal history: the command the user ran and its output.
#[derive(Debug, Clone)]
pub struct TerminalEntry {
    pub command: String,
    pub output: String,
}

#[derive(Debug)]
pub struct QuestViewState {
    pub quest_id: usize,
    /// History of (command, output) pairs — grows with each Run.
    pub history: Vec<TerminalEntry>,
    /// Terminal command input.
    pub input: String,
    pub cursor: usize,
    pub scroll_offset: usize,
    /// Current horizontal scroll offset for terminal view.
    pub h_scroll_offset: usize,
    /// The maximum possible scroll offset for the terminal view, calculated during render.
    pub max_terminal_scroll: Cell<usize>,
    /// Current scroll offset for instructions view.
    pub instructions_scroll_offset: usize,
    /// The maximum possible scroll offset for instructions view, calculated during render.
    pub max_instructions_scroll: Cell<usize>,
    /// Currently viewing history index.
    pub history_index: Option<usize>,
    /// The input typed before starting to navigate history.
    pub pending_input: String,
    /// Answer text input (for quests with submit_prompt).
    pub answer: String,
    pub answer_cursor: usize,
    /// Whether this quest has an answer input field.
    pub has_answer_input: bool,
    pub test_result: Option<TestResult>,
    pub focus: QuestFocus,
}

impl QuestViewState {
    pub fn new(quest_id: usize, has_answer_input: bool) -> Self {
        Self {
            quest_id,
            history: Vec::new(),
            input: String::new(),
            cursor: 0,
            scroll_offset: 0,
            h_scroll_offset: 0,
            max_terminal_scroll: Cell::new(0),
            instructions_scroll_offset: 0,
            max_instructions_scroll: Cell::new(0),
            history_index: None,
            pending_input: String::new(),
            answer: String::new(),
            answer_cursor: 0,
            has_answer_input,
            test_result: None,
            focus: QuestFocus::Instructions,
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
    pub global_instructions_scroll: usize,
    pub global_instructions_max_scroll: Cell<usize>,
    pub quest_view: Option<QuestViewState>,
    pub db: Db,
    pub completed: HashSet<usize>,
    pub quests: Vec<quests::Quest>,
    pub server_process: Option<Child>,
    pub exit: bool,
}

impl Drop for App {
    fn drop(&mut self) {
        self.stop_server();
    }
}

impl App {
    pub fn new() -> rusqlite::Result<Self> {
        let db = Db::open()?;
        let completed = db.completed_quests();
        let quests = quests::load_quests(std::path::Path::new("quests"));
        Ok(Self {
            sidebar_index: 0,
            reset_focused: false,
            reset_done: false,
            quest_list_index: 0,
            content_focused: false,
            global_instructions_scroll: 0,
            global_instructions_max_scroll: Cell::new(0),
            quest_view: None,
            db,
            completed,
            quests,
            server_process: None,
            exit: false,
        })
    }

    pub fn quest_count(&self) -> usize {
        self.quests.len()
    }

    pub fn get_quest(&self, id: usize) -> Option<&quests::Quest> {
        self.quests.iter().find(|q| q.id == id)
    }

    /// Run the full setup phase for a quest: seed DB, start server, create view.
    fn start_quest(&mut self, quest_id: usize) {
        let quest = match self.quests.iter().find(|q| q.id == quest_id) {
            Some(q) => q,
            None => return,
        };

        // Setup phase: wipe and seed the quest database.
        if let Some(setup) = &quest.setup {
            let _ = db::setup_quest_db(&setup.seed);
        }

        let has_answer = quest.submit_prompt.is_some();

        // Start the quest server (passes QUEST_DB env var).
        self.start_quest_server(quest_id);

        // Create the view state.
        self.quest_view = Some(QuestViewState::new(quest_id, has_answer));
    }

    fn start_quest_server(&mut self, quest_id: usize) {
        self.stop_server();
        let quest = match self.quests.iter().find(|q| q.id == quest_id) {
            Some(q) => q,
            None => return,
        };
        let script = quest.folder_path.join("server.sh");
        if script.exists() {
            let quest_db = std::env::current_dir()
                .unwrap_or_default()
                .join(db::quest_db_path());
            if let Ok(child) = std::process::Command::new("sh")
                .arg(&script)
                .env("QUEST_DB", &quest_db)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                self.server_process = Some(child);
                // Give the server a moment to bind the port.
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        }
    }

    fn stop_server(&mut self) {
        if let Some(mut child) = self.server_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Stop the server and delete the quest DB — called whenever a quest session ends.
    fn cleanup_quest(&mut self) {
        self.stop_server();
        let _ = std::fs::remove_file(db::quest_db_path());
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let _ = crossterm::execute!(io::stdout(), crossterm::event::EnableMouseCapture);
        while !self.exit {
            terminal.draw(|frame| crate::ui::draw(frame, self))?;
            self.handle_events()?;
            while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                self.handle_events()?;
                if self.exit { break; }
            }
        }
        let _ = crossterm::execute!(io::stdout(), crossterm::event::DisableMouseCapture);
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
            Event::Mouse(mouse) => {
                self.handle_mouse(mouse);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.quest_view.is_some() {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.apply_quest_action(QuestAction::ScrollUp);
                }
                MouseEventKind::ScrollDown => {
                    self.apply_quest_action(QuestAction::ScrollDown);
                }
                _ => {}
            }
        } else if self.sidebar_index == 1 {
            match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.global_instructions_scroll = self.global_instructions_scroll.saturating_sub(3);
                }
                MouseEventKind::ScrollDown => {
                    self.global_instructions_scroll = self.global_instructions_scroll
                        .saturating_add(3)
                        .min(self.global_instructions_max_scroll.get());
                }
                _ => {}
            }
        }
    }
    fn handle_key(&mut self, key: KeyEvent) {
        if self.quest_view.is_some() {
            self.handle_quest_key(key);
        } else if self.content_focused {
            match self.sidebar_index {
                0 => self.handle_list_key(key),
                1 => self.handle_global_instructions_key(key),
                _ => { self.content_focused = false; }
            }
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
                        1 => self.content_focused = true,
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
        let total = self.quest_count();
        if total == 0 {
            return;
        }
        let last = total - 1;
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
                if next < total {
                    self.quest_list_index = next;
                } else if self.quest_list_index / cols < last / cols {
                    self.quest_list_index = last;
                }
            }
            KeyCode::Left => {
                if self.quest_list_index > 0 {
                    self.quest_list_index -= 1;
                }
            }
            KeyCode::Right => {
                if self.quest_list_index < last {
                    self.quest_list_index += 1;
                }
            }
            KeyCode::Enter => {
                let id = self.quest_list_index + 1;
                self.start_quest(id);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Global instructions (sidebar tab) key handling
    // -----------------------------------------------------------------------

    fn handle_global_instructions_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc | KeyCode::Left | KeyCode::Backspace => self.content_focused = false,
            KeyCode::Up => {
                self.global_instructions_scroll = self.global_instructions_scroll.saturating_sub(1);
            }
            KeyCode::Down => {
                self.global_instructions_scroll = self.global_instructions_scroll
                    .saturating_add(1)
                    .min(self.global_instructions_max_scroll.get());
            }
            KeyCode::PageUp => {
                self.global_instructions_scroll = self.global_instructions_scroll.saturating_sub(15);
            }
            KeyCode::PageDown => {
                self.global_instructions_scroll = self.global_instructions_scroll
                    .saturating_add(15)
                    .min(self.global_instructions_max_scroll.get());
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
        let action = {
            let qv = self.quest_view.as_ref().unwrap();
            resolve_quest_action(qv, key)
        };
        self.apply_quest_action(action);
    }

    fn apply_quest_action(&mut self, action: QuestAction) {
        match action {
            QuestAction::None => {}

            // Terminal input editing
            QuestAction::Insert(c) => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.input.insert(qv.cursor, c);
                qv.cursor += c.len_utf8();
                qv.scroll_offset = 0;
            }
            QuestAction::Backspace => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor > 0 {
                    if let Some(c) = qv.input[..qv.cursor].chars().next_back() {
                        qv.cursor -= c.len_utf8();
                        qv.input.remove(qv.cursor);
                    }
                }
            }
            QuestAction::CursorLeft => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor > 0 {
                    if let Some(c) = qv.input[..qv.cursor].chars().next_back() {
                        qv.cursor -= c.len_utf8();
                    }
                }
            }
            QuestAction::CursorRight => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.cursor < qv.input.len() {
                    if let Some(c) = qv.input[qv.cursor..].chars().next() {
                        qv.cursor += c.len_utf8();
                    }
                }
            }
            QuestAction::PageUp => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.focus == QuestFocus::Instructions {
                    qv.instructions_scroll_offset = qv.instructions_scroll_offset.saturating_sub(15);
                } else {
                    qv.scroll_offset = qv.scroll_offset.saturating_add(15).min(qv.max_terminal_scroll.get());
                }
            }
            QuestAction::PageDown => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.focus == QuestFocus::Instructions {
                    qv.instructions_scroll_offset = qv.instructions_scroll_offset.saturating_add(15).min(qv.max_instructions_scroll.get());
                } else {
                    qv.scroll_offset = qv.scroll_offset.saturating_sub(15);
                }
            }
            QuestAction::ScrollUp => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.focus == QuestFocus::Instructions {
                    qv.instructions_scroll_offset = qv.instructions_scroll_offset.saturating_sub(3);
                } else {
                    qv.scroll_offset = qv.scroll_offset.saturating_add(3).min(qv.max_terminal_scroll.get());
                }
            }
            QuestAction::ScrollDown => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.focus == QuestFocus::Instructions {
                    qv.instructions_scroll_offset = qv.instructions_scroll_offset.saturating_add(3).min(qv.max_instructions_scroll.get());
                } else {
                    qv.scroll_offset = qv.scroll_offset.saturating_sub(3);
                }
            }
            QuestAction::HistoryUp => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.history.is_empty() { return; }
                
                if qv.history_index.is_none() {
                    qv.pending_input = qv.input.clone();
                    qv.history_index = Some(qv.history.len().saturating_sub(1));
                } else {
                    let idx = qv.history_index.unwrap();
                    if idx > 0 {
                        qv.history_index = Some(idx - 1);
                    }
                }
                
                if let Some(idx) = qv.history_index {
                    qv.input = qv.history[idx].command.clone();
                    qv.cursor = qv.input.len();
                }
            }
            QuestAction::HistoryDown => {
                let qv = self.quest_view.as_mut().unwrap();
                if let Some(idx) = qv.history_index {
                    if idx + 1 < qv.history.len() {
                        qv.history_index = Some(idx + 1);
                        qv.input = qv.history[idx + 1].command.clone();
                    } else {
                        qv.history_index = None;
                        qv.input = qv.pending_input.clone();
                    }
                    qv.cursor = qv.input.len();
                }
            }

            // Answer input editing
            QuestAction::AnswerInsert(c) => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.answer.insert(qv.answer_cursor, c);
                qv.answer_cursor += 1;
            }
            QuestAction::AnswerBackspace => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.answer_cursor > 0 {
                    qv.answer_cursor -= 1;
                    qv.answer.remove(qv.answer_cursor);
                }
            }
            QuestAction::AnswerCursorLeft => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.answer_cursor > 0 {
                    qv.answer_cursor -= 1;
                }
            }
            QuestAction::AnswerCursorRight => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.answer_cursor < qv.answer.len() {
                    qv.answer_cursor += 1;
                }
            }

            QuestAction::Enter => {
                let qv = self.quest_view.as_mut().unwrap();
                let is_multiline = qv.input.trim_end().ends_with('\\');
                if is_multiline {
                    qv.input.insert(qv.cursor, '\n');
                    qv.cursor += 1;
                    qv.scroll_offset = 0;
                } else {
                    self.run_command();
                }
            }
            QuestAction::Submit => self.submit_quest(),

            QuestAction::FocusTerminal => {
                self.quest_view.as_mut().unwrap().focus = QuestFocus::Terminal;
            }
            QuestAction::FocusNext => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.focus = qv.focus.next(qv.has_answer_input);
            }
            QuestAction::FocusPrev => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.focus = qv.focus.prev(qv.has_answer_input);
            }
            QuestAction::Back => {
                self.cleanup_quest();
                self.quest_view = None;
                self.content_focused = true;
            }
            QuestAction::Escape => {
                let qv = self.quest_view.as_mut().unwrap();
                if qv.test_result.is_some() {
                    qv.test_result = None;
                } else {
                    self.cleanup_quest();
                    self.quest_view = None;
                    self.content_focused = true;
                }
            }

            QuestAction::Paste => {
                if let Ok(out) = std::process::Command::new("pbpaste").output() {
                    let text = String::from_utf8_lossy(&out.stdout).to_string();
                    let qv = self.quest_view.as_mut().unwrap();
                    for c in text.chars() {
                        if c == '\r' { continue; }
                        qv.input.insert(qv.cursor, c);
                        qv.cursor += c.len_utf8();
                    }
                    qv.scroll_offset = 0;
                }
            }

            QuestAction::HScrollLeft => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.h_scroll_offset = qv.h_scroll_offset.saturating_sub(8);
            }
            QuestAction::HScrollRight => {
                let qv = self.quest_view.as_mut().unwrap();
                qv.h_scroll_offset = qv.h_scroll_offset.saturating_add(8);
            }

            QuestAction::CopyLastOutput => {
                let qv = self.quest_view.as_ref().unwrap();
                if let Some(last) = qv.history.last() {
                    let output = last.output.clone();
                    let mut child = std::process::Command::new("pbcopy")
                        .stdin(std::process::Stdio::piped())
                        .spawn()
                        .ok();
                    if let Some(ref mut c) = child {
                        if let Some(stdin) = c.stdin.as_mut() {
                            let _ = std::io::Write::write_all(stdin, output.as_bytes());
                        }
                        let _ = c.wait();
                    }
                }
            }
        }
    }

    fn run_command(&mut self) {
        let qv = self.quest_view.as_mut().unwrap();
        let cmd = qv.input.trim().to_string();
        if cmd.is_empty() {
            return;
        }
        let result = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output();
        let output = match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if !stdout.is_empty() && !stderr.is_empty() {
                    format!("{}\n---stderr---\n{}", stdout, stderr)
                } else if !stdout.is_empty() {
                    stdout
                } else if !stderr.is_empty() {
                    stderr
                } else {
                    "(no output — is the server running? try again in a moment)".to_string()
                }
            }
            Err(e) => format!("Failed to run command: {}", e),
        };
        qv.history.push(TerminalEntry { command: cmd, output });
        qv.input.clear();
        qv.cursor = 0;
        qv.scroll_offset = 0;
        qv.h_scroll_offset = 0;
        qv.history_index = None;
        qv.pending_input.clear();
        qv.test_result = None;
    }

    fn submit_quest(&mut self) {
        let quest_id = self.quest_view.as_ref().unwrap().quest_id;
        let answer = self.quest_view.as_ref().unwrap().answer.clone();

        let quest = match self.quests.iter().find(|q| q.id == quest_id) {
            Some(q) => q,
            None => return,
        };

        // 1. Run DB verification checks.
        let db_result = quest.verify_db(&db::quest_db_path());
        if let quests::VerifyResult::Fail(msg) = db_result {
            self.quest_view.as_mut().unwrap().test_result = Some(TestResult::Fail(msg));
            return;
        }

        // 2. Run input verification (if quest requires it).
        let input_result = quest.verify_input(&answer);
        if let quests::VerifyResult::Fail(msg) = input_result {
            self.quest_view.as_mut().unwrap().test_result = Some(TestResult::Fail(msg));
            return;
        }

        // All checks passed — mark complete, stop server, clean DB.
        self.completed.insert(quest_id);
        let _ = self.db.mark_completed(quest_id);
        self.cleanup_quest();
        self.quest_view.as_mut().unwrap().test_result = Some(TestResult::Pass);
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
    PageUp,
    PageDown,
    ScrollUp,
    ScrollDown,
    HistoryUp,
    HistoryDown,
    AnswerInsert(char),
    AnswerBackspace,
    AnswerCursorLeft,
    AnswerCursorRight,
    Enter,
    Submit,
    FocusTerminal,
    FocusNext,
    FocusPrev,
    Back,
    Escape,
    Paste,
    CopyLastOutput,
    HScrollLeft,
    HScrollRight,
}

fn resolve_quest_action(qv: &QuestViewState, key: KeyEvent) -> QuestAction {
    match key.code {
        KeyCode::Esc => QuestAction::Escape,
        _ => match qv.focus {
            QuestFocus::Instructions => match key.code {
                KeyCode::Up => QuestAction::ScrollUp,
                KeyCode::Down => QuestAction::ScrollDown,
                KeyCode::PageUp => QuestAction::PageUp,
                KeyCode::PageDown => QuestAction::PageDown,
                KeyCode::Tab => QuestAction::FocusNext,
                KeyCode::BackTab => QuestAction::FocusPrev,
                KeyCode::Enter => QuestAction::FocusNext,
                _ => QuestAction::None,
            },
            QuestFocus::Terminal => match key.code {
                KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => QuestAction::Paste,
                KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => QuestAction::CopyLastOutput,
                KeyCode::Char(c) => QuestAction::Insert(c),
                KeyCode::Backspace => QuestAction::Backspace,
                KeyCode::Left => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        QuestAction::HScrollLeft
                    } else {
                        QuestAction::CursorLeft
                    }
                }
                KeyCode::Right => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        QuestAction::HScrollRight
                    } else {
                        QuestAction::CursorRight
                    }
                }
                KeyCode::PageUp => QuestAction::PageUp,
                KeyCode::PageDown => QuestAction::PageDown,
                KeyCode::Up => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        QuestAction::ScrollUp
                    } else {
                        QuestAction::HistoryUp
                    }
                }
                KeyCode::Down => {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        QuestAction::ScrollDown
                    } else {
                        QuestAction::HistoryDown
                    }
                }
                KeyCode::Enter => QuestAction::Enter,
                KeyCode::Tab => QuestAction::FocusNext,
                KeyCode::BackTab => QuestAction::FocusPrev,
                _ => QuestAction::None,
            },
            QuestFocus::Answer => match key.code {
                KeyCode::Char(c) => QuestAction::AnswerInsert(c),
                KeyCode::Backspace => QuestAction::AnswerBackspace,
                KeyCode::Left => QuestAction::AnswerCursorLeft,
                KeyCode::Right => QuestAction::AnswerCursorRight,
                KeyCode::Tab => QuestAction::FocusNext,
                KeyCode::BackTab => QuestAction::FocusPrev,
                KeyCode::Enter => QuestAction::FocusNext,
                _ => QuestAction::None,
            },
            QuestFocus::Submit => match key.code {
                KeyCode::Enter => QuestAction::Submit,
                KeyCode::Tab | KeyCode::Right => QuestAction::FocusNext,
                KeyCode::BackTab | KeyCode::Left => QuestAction::FocusPrev,
                KeyCode::Char(_) => QuestAction::FocusTerminal,
                _ => QuestAction::None,
            },
            QuestFocus::Back => match key.code {
                KeyCode::Enter => QuestAction::Back,
                KeyCode::Tab | KeyCode::Right => QuestAction::FocusNext,
                KeyCode::BackTab | KeyCode::Left => QuestAction::FocusPrev,
                KeyCode::Char(_) => QuestAction::FocusTerminal,
                _ => QuestAction::None,
            },
        },
    }
}
