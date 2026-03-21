pub mod levels;
pub mod quest_view;
pub mod sidebar;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks =
        Layout::horizontal([Constraint::Length(22), Constraint::Fill(1)]).split(frame.area());

    sidebar::render(frame, app, chunks[0]);

    if let Some(qv) = &app.quest_view {
        quest_view::render(frame, app, qv, chunks[1]);
    } else if app.reset_focused {
        render_reset(frame, app, chunks[1]);
    } else {
        match app.sidebar_index {
            0 => levels::render(frame, app, chunks[1]),
            1 => render_instructions(frame, chunks[1]),
            2 => render_exit(frame, chunks[1]),
            _ => {}
        }
    }
}

fn render_instructions(frame: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::bordered()
        .title(" Instructions ")
        .border_style(Style::new().fg(Color::Cyan));

    frame.render_widget(
        Paragraph::new(
            "How to play:\n\n\
             1. Go to Levels and select a Quest\n\
             2. Read the instructions in the quest view\n\
             3. Type a curl command in the Command box\n\
             4. Press Enter or [ Run ] to execute it\n\
             5. Press [ Submit ] to check your answer\n\n\
             Navigation:\n\
               Up / Down     Move sidebar selection\n\
               Enter         Open / activate\n\
               Tab / ← →     Switch between buttons\n\
               Esc           Go back\n\
               q             Quit\n\n\
             Completed quests show green in the grid.\n\
             Use Reset to clear all progress.",
        )
        .block(block)
        .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_reset(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    use ratatui::layout::Alignment;

    let (title, body, color) = if app.reset_done {
        (
            " Reset ",
            "✓  Progress reset.\n\nAll completed quests have been cleared.\nThe grid is now empty — good luck!",
            Color::Green,
        )
    } else {
        (
            " Reset Progress ",
            "This will clear all your completed quest progress.\n\nThe quest grid will go back to zero — no quests marked complete.\n\nPress Enter to confirm.",
            Color::Yellow,
        )
    };

    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(color));

    frame.render_widget(
        Paragraph::new(body)
            .style(Style::new().fg(color))
            .alignment(Alignment::Left)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_exit(frame: &mut Frame, area: ratatui::layout::Rect) {
    let block = Block::bordered()
        .title(" Exit ")
        .border_style(Style::new().fg(Color::Cyan));

    frame.render_widget(
        Paragraph::new("Press Enter to exit the application.")
            .bold()
            .block(block),
        area,
    );
}
