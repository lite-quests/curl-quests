use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::app::{App, QuestFocus, QuestViewState, TestResult};

pub fn render(frame: &mut Frame, app: &App, qv: &QuestViewState, area: Rect) {
    let quest = match app.get_quest(qv.quest_id) {
        Some(q) => q,
        None => return,
    };

    let done_badge = if app.completed.contains(&qv.quest_id) { " ✓" } else { "" };
    let title = format!(
        " Quest {}/{}: {}{} ",
        qv.quest_id,
        app.quest_count(),
        quest.title,
        done_badge
    );

    let outer = Block::bordered()
        .title(title.as_str())
        .border_style(Style::new().fg(Color::Cyan));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Build layout dynamically — answer input row only when the quest needs it.
    let mut constraints = vec![
        Constraint::Percentage(30), // instructions + hint
        Constraint::Fill(1),        // terminal (history + live input)
    ];
    if qv.has_answer_input {
        constraints.push(Constraint::Length(3)); // answer input
    }
    constraints.push(Constraint::Length(3)); // submit result
    constraints.push(Constraint::Length(3)); // buttons

    let chunks = Layout::vertical(constraints).split(inner);

    let mut idx = 0;
    render_instructions(frame, qv, &quest.instructions, &quest.hint, chunks[idx]);
    idx += 1;
    render_terminal(frame, qv, chunks[idx]);
    idx += 1;
    if qv.has_answer_input {
        let prompt = quest.submit_prompt.as_deref().unwrap_or("Your answer");
        render_answer_input(frame, qv, prompt, chunks[idx]);
        idx += 1;
    }
    render_result(frame, qv, chunks[idx]);
    idx += 1;
    render_buttons(frame, qv, chunks[idx]);
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

fn render_instructions(frame: &mut Frame, qv: &QuestViewState, instructions: &str, hint: &str, area: Rect) {
    let focused = qv.focus == QuestFocus::Instructions;
    let border_color = if focused { Color::Yellow } else { Color::DarkGray };
    let title = if focused {
        " Instructions — ↑↓ scroll  Tab next "
    } else {
        " Instructions — ↑↓ to scroll "
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(border_color));
    let text = format!("{}\n\nHint: {}", instructions, hint);

    // Count source lines and estimate display lines accounting for word wrap.
    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let inner_height = area.height.saturating_sub(2);
    let total_display_lines: usize = text.lines().map(|l| {
        let chars = l.chars().count();
        // Rough word-wrap estimate: assume ~80% fill efficiency due to word boundaries
        ((chars * 10 / 8 + inner_width - 1) / inner_width).max(1)
    }).sum();
    let max_scroll = (total_display_lines as u16).saturating_sub(inner_height);
    let scroll = (qv.instructions_scroll_offset as u16).min(max_scroll);

    frame.render_widget(
        Paragraph::new(text.clone()).block(block).wrap(Wrap { trim: false }).scroll((scroll, 0)),
        area,
    );

    // Scrollbar on the right edge (always shown so user knows content is scrollable)
    let scrollbar_area = Rect {
        x: area.x + area.width - 1,
        y: area.y + 1,
        width: 1,
        height: area.height.saturating_sub(2),
    };
    let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
        .position(scroll as usize);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        scrollbar_area,
        &mut scrollbar_state,
    );
}

fn render_terminal(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let focused = qv.focus == QuestFocus::Terminal;
    let border_color = if focused { Color::Yellow } else { Color::DarkGray };
    let title = if focused {
        " Terminal (Enter to run  Tab to navigate  Shift+↑↓ scroll) "
    } else {
        " Terminal "
    };

    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(border_color));
    let inner = block.inner(area);

    // Build lines: history entries, then the live input prompt.
    let mut lines: Vec<Line> = Vec::new();

    for entry in &qv.history {
        // Command line(s)
        let mut first = true;
        for line in entry.command.lines() {
            let prompt = if first { "$ " } else { "> " };
            first = false;
            lines.push(Line::from(vec![
                Span::styled(prompt, Style::new().fg(Color::Green).bold()),
                Span::styled(line.to_string(), Style::new().fg(Color::White)),
            ]));
        }
        // Output lines
        for output_line in entry.output.lines() {
            lines.push(Line::from(Span::styled(
                format!("  {}", output_line),
                Style::new().fg(Color::Gray),
            )));
        }
        // Blank separator
        lines.push(Line::default());
    }

    // Live input prompt at the bottom
    let mut spans = vec![Span::styled("$ ", Style::new().fg(Color::Green).bold())];
    
    for (i, c) in qv.input.char_indices() {
        let is_cursor = i == qv.cursor && focused;
        let style = if is_cursor { Style::new().fg(Color::Black).bg(Color::White) } else { Style::new() };
        
        if c == '\n' {
            if is_cursor {
                spans.push(Span::styled("█", Style::new().fg(Color::Black).bg(Color::White)));
            }
            lines.push(Line::from(spans));
            spans = vec![Span::styled("> ", Style::new().fg(Color::Green).bold())];
        } else {
            spans.push(Span::styled(c.to_string(), style));
        }
    }
    
    if qv.cursor == qv.input.len() {
        spans.push(Span::styled(
            if focused { "█" } else { "" },
            Style::new().bg(Color::White).fg(Color::Black)
        ));
    }
    lines.push(Line::from(spans));

    // Scroll so the bottom (live input) is always visible, accounting for user scroll_offset.
    let total_lines = lines.len() as u16;
    let visible = inner.height;
    let max_scroll = total_lines.saturating_sub(visible);
    let effective_scroll_offset = qv.scroll_offset.min(max_scroll as usize) as u16;
    let scroll = max_scroll.saturating_sub(effective_scroll_offset);

    frame.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll, 0)),
        area,
    );
}

fn render_answer_input(frame: &mut Frame, qv: &QuestViewState, prompt: &str, area: Rect) {
    let focused = qv.focus == QuestFocus::Answer;
    let border_color = if focused { Color::Yellow } else { Color::DarkGray };
    let title = format!(" {} ", prompt);

    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(border_color));

    let before = &qv.answer[..qv.answer_cursor];
    let after = &qv.answer[qv.answer_cursor..];
    let line = Line::from(vec![
        Span::styled("> ", Style::new().fg(Color::Cyan).bold()),
        Span::raw(before.to_string()),
        Span::styled(
            if focused { "█" } else { "" },
            Style::new().fg(Color::Black).bg(Color::White),
        ),
        Span::raw(after.to_string()),
    ]);

    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn render_result(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let (text, color) = match &qv.test_result {
        Some(TestResult::Pass) => ("  ✓  Quest complete! Well done.", Color::Green),
        Some(TestResult::Fail(msg)) => (msg.as_str(), Color::Red),
        None => (
            "  Press Tab → [Submit] to verify your work.",
            Color::DarkGray,
        ),
    };
    let focused = qv.focus == QuestFocus::Submit;
    let border_color = if focused { Color::Yellow } else { color };
    let block = Block::bordered().border_style(Style::new().fg(border_color));
    frame.render_widget(
        Paragraph::new(text)
            .style(Style::new().fg(color).bold())
            .block(block),
        area,
    );
}

fn render_buttons(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let sel = Style::new().bg(Color::Yellow).fg(Color::Black).bold();
    let normal = Style::new().fg(Color::White);

    let sub_s = if qv.focus == QuestFocus::Submit { sel } else { normal };
    let back_s = if qv.focus == QuestFocus::Back { sel } else { normal };

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("[ Submit ]", sub_s),
        Span::raw("   "),
        Span::styled("[ Back ]", back_s),
        Span::raw("   "),
        Span::styled(
            "Tab/←→ switch  Esc back",
            Style::new().fg(Color::DarkGray),
        ),
    ]);

    let block = Block::bordered().border_style(Style::new().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(line).block(block), area);
}
