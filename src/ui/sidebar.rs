use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::app::{App, SIDEBAR_ITEMS};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(area);

    // Nav list
    let list_items: Vec<ListItem> = SIDEBAR_ITEMS.iter().map(|s| ListItem::new(*s)).collect();

    let mut state = ListState::default();
    if !app.reset_focused {
        state.select(Some(app.sidebar_index));
    }

    let list = List::new(list_items)
        .block(
            Block::bordered()
                .title(" curl-quests ")
                .border_style(Style::new().fg(Color::Cyan)),
        )
        .highlight_style(Style::new().bg(Color::Cyan).fg(Color::Black).bold());

    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Reset button
    let (btn_style, border_style) = if app.reset_focused {
        (
            Style::new().bg(Color::Yellow).fg(Color::Black).bold(),
            Style::new().fg(Color::Yellow),
        )
    } else {
        (Style::new().fg(Color::Gray), Style::new().fg(Color::Gray))
    };

    frame.render_widget(
        Paragraph::new("  Reset")
            .style(btn_style)
            .block(Block::bordered().border_style(border_style)),
        chunks[1],
    );
}
