use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph, Wrap},
};

const SIDEBAR: [&str; 3] = ["  Levels", "  Instructions", "  Exit"];

#[derive(Debug, Default)]
pub struct App {
    sidebar_index: usize,
    reset_focused: bool,
    content_focused: bool,
    quest_index: usize,
    selected_quest: Option<usize>,
    exit: bool,
}

fn main() -> io::Result<()> {
    ratatui::run(|terminal| App::default().run(terminal))
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks =
            Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)]).split(frame.area());
        self.render_sidebar(frame, chunks[0]);
        self.render_content(frame, chunks[1]);
    }

    fn render_sidebar(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(area);

        let list_items: Vec<ListItem> = SIDEBAR.iter().map(|s| ListItem::new(*s)).collect();

        let mut list_state = ListState::default();
        if !self.reset_focused {
            list_state.select(Some(self.sidebar_index));
        }

        let list = List::new(list_items)
            .block(
                Block::bordered()
                    .title(" curl-quests ")
                    .border_style(Style::new().fg(Color::Cyan)),
            )
            .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black).bold());

        frame.render_stateful_widget(list, chunks[0], &mut list_state);

        let (reset_style, reset_border) = if self.reset_focused {
            (
                Style::new().bg(Color::Yellow).fg(Color::Black).bold(),
                Style::new().fg(Color::Yellow),
            )
        } else {
            (Style::new().fg(Color::Gray), Style::new().fg(Color::Gray))
        };

        frame.render_widget(
            Paragraph::new("  Reset")
                .style(reset_style)
                .block(Block::bordered().border_style(reset_border)),
            chunks[1],
        );
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        match self.sidebar_index {
            0 => self.render_levels(frame, area),
            1 => {
                let block = Block::bordered()
                    .title(" Instructions - Enter to select")
                    .border_style(Style::new().fg(Color::Cyan));
                frame.render_widget(
                    Paragraph::new(
                        "How to play:\n\n\
                         1. Go to Levels and select a Quest\n\
                         2. Run the curl command shown\n\
                         3. Follow the instructions in the response\n\n\
                         Navigation:\n\
                           Arrows      Move selection\n\
                           Enter       Select / confirm\n\
                           Esc         Go back\n\
                           q           Quit\n\n\
                         Use Reset to clear your progress.",
                    )
                    .block(block)
                    .wrap(Wrap { trim: false }),
                    area,
                );
            }
            2 => {
                let block = Block::bordered()
                    .title(" Exit ")
                    .border_style(Style::new().fg(Color::Cyan));
                frame.render_widget(
                    Paragraph::new("Press Enter to exit the application.").block(block),
                    area,
                );
            }
            _ => {}
        }
    }

    fn render_levels(&self, frame: &mut Frame, area: Rect) {
        if let Some(n) = self.selected_quest {
            // Quest detail
            let block = Block::bordered()
                .title(format!(" Quest {n} of 24 -  Esc to exit "))
                .border_style(Style::new().fg(Color::Cyan));
            frame.render_widget(
                Paragraph::new(format!(
                    "Quest {n} of 24\n\n\
                     Complete the curl challenge:\n\n\
                       curl https://example.com/quest{n}\n\n\
                     Read the response and follow the instructions.\n\n\
                     Press Esc to return to the quest list."
                ))
                .block(block)
                .wrap(Wrap { trim: false }),
                area,
            );
        } else {
            // Quest grid
            let title = if self.content_focused {
                " Levels - Esc to exit "
            } else {
                " Levels - Enter to select "
            };

            let outer_block = Block::bordered()
                .title(title)
                .border_style(Style::new().fg(Color::Cyan));
            let inner_area = outer_block.inner(area);
            frame.render_widget(outer_block, area);

            let box_width = 6;
            let box_height = 3;
            let spacing_x = 2;
            let spacing_y = 1;

            let cols =
                ((inner_area.width as usize + spacing_x) / (box_width as usize + spacing_x)).max(1);
            let total_rows = (24 + cols - 1) / cols;
            let visible_rows = ((inner_area.height as usize + spacing_y)
                / (box_height as usize + spacing_y))
                .max(1);

            let selected_row = self.quest_index / cols;

            let mut start_row = selected_row.saturating_sub(visible_rows / 2);
            if start_row + visible_rows > total_rows {
                start_row = total_rows.saturating_sub(visible_rows);
            }

            for i in 0..24 {
                let row = i / cols;
                let col = i % cols;

                if row < start_row || row >= start_row + visible_rows {
                    continue;
                }

                let visual_row = row - start_row;

                let x = inner_area.x + (col * (box_width as usize + spacing_x)) as u16;
                let y = inner_area.y + (visual_row * (box_height as usize + spacing_y)) as u16;

                let rect = Rect::new(x, y, box_width as u16, box_height as u16);
                let rect = inner_area.intersection(rect);

                if rect.width > 0 && rect.height > 0 {
                    let is_selected = self.content_focused && self.quest_index == i;
                    let block = Block::bordered();

                    let (bg, fg, border_color) = if is_selected {
                        (Color::Blue, Color::White, Color::Blue)
                    } else {
                        (Color::Reset, Color::White, Color::DarkGray)
                    };

                    let cell_block = block
                        .border_style(Style::new().fg(border_color))
                        .style(Style::new().bg(bg).fg(fg));

                    let paragraph = Paragraph::new(format!("{}", i + 1))
                        .block(cell_block)
                        .alignment(ratatui::layout::Alignment::Center);

                    frame.render_widget(paragraph, rect);
                }
            }
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let cols = if let Ok(size) = crossterm::terminal::size() {
                    let area_width = size.0.saturating_sub(22).saturating_sub(2);
                    ((area_width + 2) / 16).max(1) as usize
                } else {
                    4
                };
                self.handle_key(key, cols);
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent, cols: usize) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Esc => self.handle_esc(),
            KeyCode::Up => self.handle_up(cols),
            KeyCode::Down => self.handle_down(cols),
            KeyCode::Left => self.handle_left(),
            KeyCode::Right => self.handle_right(),
            KeyCode::Enter => self.handle_enter(),
            _ => {}
        }
    }

    fn handle_esc(&mut self) {
        if self.selected_quest.is_some() {
            self.selected_quest = None;
            self.content_focused = true;
        } else if self.content_focused {
            self.content_focused = false;
        } else {
            self.exit = true;
        }
    }

    fn handle_up(&mut self, cols: usize) {
        if self.content_focused {
            if self.quest_index >= cols {
                self.quest_index -= cols;
            }
        } else if self.reset_focused {
            self.reset_focused = false;
        } else if self.sidebar_index > 0 {
            self.set_sidebar(self.sidebar_index - 1);
        }
    }

    fn handle_down(&mut self, cols: usize) {
        if self.content_focused {
            let next = self.quest_index + cols;
            if next < 24 {
                self.quest_index = next;
            } else if self.quest_index / cols < 23 / cols {
                self.quest_index = 23;
            }
        } else if !self.reset_focused {
            if self.sidebar_index < SIDEBAR.len() - 1 {
                self.set_sidebar(self.sidebar_index + 1);
            } else {
                self.reset_focused = true;
            }
        }
    }

    fn handle_left(&mut self) {
        if self.content_focused {
            if self.quest_index > 0 {
                self.quest_index -= 1;
            }
        }
    }

    fn handle_right(&mut self) {
        if self.content_focused {
            if self.quest_index < 23 {
                self.quest_index += 1;
            }
        }
    }

    fn handle_enter(&mut self) {
        if self.reset_focused {
            self.selected_quest = None;
            self.quest_index = 0;
        } else if self.content_focused {
            self.selected_quest = Some(self.quest_index + 1);
        } else {
            match self.sidebar_index {
                0 => self.content_focused = true,
                2 => self.exit = true,
                _ => {}
            }
        }
    }

    fn set_sidebar(&mut self, index: usize) {
        self.sidebar_index = index;
        self.content_focused = false;
        self.selected_quest = None;
    }
}
