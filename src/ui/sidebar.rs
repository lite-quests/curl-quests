use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Paragraph, Tabs},
    layout::Alignment,
};

use crate::app::{App, SIDEBAR_ITEMS};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(10),
        Constraint::Length(30),
    ])
    .split(area);

    let mut tabs = Tabs::new(SIDEBAR_ITEMS.iter().copied())
        .block(Block::bordered().title(" curl-quests ").border_style(Style::new().fg(Color::Cyan)));

    if !app.reset_focused {
        tabs = tabs
            .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black).bold())
            .select(app.sidebar_index);
    }

    frame.render_widget(tabs, chunks[0]);

    let (btn_style, border_style) = if app.reset_focused {
        (
            Style::new().bg(Color::Yellow).fg(Color::Black).bold(),
            Style::new().fg(Color::Yellow),
        )
    } else {
        (Style::new().fg(Color::Gray), Style::new().fg(Color::Gray))
    };

    frame.render_widget(
        Paragraph::new(" Reset ")
            .alignment(Alignment::Center)
            .style(btn_style)
            .block(Block::bordered().border_style(border_style)),
        chunks[1],
    );

    let text = ratatui::text::Line::from(vec![
        ratatui::text::Span::raw("Powered by "),
        ratatui::text::Span::styled("⚡ Lite Quests", Style::new().fg(Color::Yellow).bold()),
    ]);

    frame.render_widget(
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::bordered().border_style(Style::new().fg(Color::DarkGray))),
        chunks[2],
    );
}

