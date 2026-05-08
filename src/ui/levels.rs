use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let title = if app.content_focused {
        " Levels: ↑↓←→ navigate  Enter open  Esc back "
    } else {
        " Levels:Enter to browse quests "
    };

    let outer = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(Color::Cyan));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let total = app.quest_count();
    if total == 0 {
        return;
    }

    let box_w: u16 = 32;
    let box_h: u16 = 3;
    let gap_x: u16 = 1;
    let gap_y: u16 = 1;

    let cols = ((inner.width + gap_x) / (box_w + gap_x)).max(1) as usize;
    let total_rows = (total + cols - 1) / cols;
    let vis_rows = ((inner.height + gap_y) / (box_h + gap_y)).max(1) as usize;

    let sel_row = app.quest_list_index / cols;
    let mut start_row = sel_row.saturating_sub(vis_rows / 2);
    if start_row + vis_rows > total_rows {
        start_row = total_rows.saturating_sub(vis_rows);
    }

    for i in 0..total {
        let row = i / cols;
        let col = i % cols;

        if row < start_row || row >= start_row + vis_rows {
            continue;
        }
        let vis_row = row - start_row;

        let x = inner.x + (col as u16) * (box_w + gap_x);
        let y = inner.y + (vis_row as u16) * (box_h + gap_y);
        let rect = Rect::new(x, y, box_w, box_h).intersection(inner);

        if rect.width == 0 || rect.height == 0 {
            continue;
        }

        let quest_id = i + 1;
        let is_sel = app.content_focused && app.quest_list_index == i;
        let is_done = app.completed.contains(&quest_id);

        let (bg, fg, border_color) = if is_sel {
            (Color::Blue, Color::White, Color::Blue)
        } else if is_done {
            (Color::Reset, Color::Green, Color::Green)
        } else {
            (Color::Reset, Color::DarkGray, Color::DarkGray)
        };

        let title = app.get_quest(quest_id).map(|q| q.title.as_str()).unwrap_or("");
        let prefix = format!(" {}. {}", quest_id, title);
        let label = if is_done {
            format!("{} ✓", prefix)
        } else {
            prefix
        };

        let block = Block::bordered()
            .border_style(Style::new().fg(border_color))
            .style(Style::new().bg(bg).fg(fg));

        frame.render_widget(
            Paragraph::new(label)
                .block(block)
                .bold()
                .alignment(Alignment::Left),
            rect,
        );
    }
}
