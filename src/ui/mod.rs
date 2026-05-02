pub mod levels;
pub mod quest_view;
pub mod sidebar;

use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let chunks =
        Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(frame.area());

    sidebar::render(frame, app, chunks[0]);

    if let Some(qv) = &app.quest_view {
        quest_view::render(frame, app, qv, chunks[1]);
    } else if app.reset_focused {
        render_reset(frame, app, chunks[1]);
    } else {
        match app.sidebar_index {
            0 => levels::render(frame, app, chunks[1]),
            1 => render_instructions(frame, app, chunks[1]),
            2 => render_exit(frame, chunks[1]),
            _ => {}
        }
    }
}

fn count_wrapped_lines(text: &str, width: u16) -> u16 {
    let mut count = 0;
    let w = width.max(1) as usize;
    for line in text.lines() {
        let len = line.chars().count();
        if len == 0 {
            count += 1;
        } else {
            count += ((len - 1) / w) as u16 + 1;
        }
    }
    count
}

fn render_instructions(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let is_focused = app.content_focused && app.sidebar_index == 1;
    let border_color = if is_focused { Color::Yellow } else { Color::Cyan };
    let block = Block::bordered()
        .title(" Instructions ")
        .border_style(Style::new().fg(border_color));

    let text = "How to play:\n\n\
             1. Go to Levels and select a Quest\n\
             2. The quest database is seeded and a server starts automatically\n\
             3. Read the instructions and use the Terminal to run curl commands\n\
             4. If the quest asks for an answer, type it in the Answer box\n\
             5. Press [ Submit ] to verify — checks run against the database\n\
             6. If you fail, try again! The quest stays open until you pass\n\n\
             General Navigation:\n\
               Left / Right  Move tab selection\n\
               Enter         Open / activate\n\
               Tab / ← →     Switch focus between sections\n\
               Esc           Go back / dismiss result\n\
               q             Quit\n\n\
             Quest Controls:\n\
               Up / Down     (Instructions) Scroll text up / down\n\
               Up / Down     (Terminal) Navigate through previous commands\n\
               Shift + ↑/↓   (Terminal) Scroll terminal output up / down\n\
               PageUp/Down   Scroll up / down quickly\n\n\
             Completed quests show green in the grid.\n\
             Use Reset to clear all progress.";

    let inner = block.inner(area);
    let total_lines = count_wrapped_lines(text, inner.width);
    let max_scroll = total_lines.saturating_sub(inner.height);
    app.global_instructions_max_scroll.set(max_scroll as usize);
    let scroll = app.global_instructions_scroll.min(max_scroll as usize) as u16;

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        area,
    );

    if max_scroll > 0 {
        let mut scrollbar_state = ScrollbarState::default()
            .content_length(max_scroll as usize)
            .position(scroll as usize);
            
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            area,
            &mut scrollbar_state,
        );
    }
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
