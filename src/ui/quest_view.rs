use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

use crate::app::{App, QuestFocus, QuestViewState, TestResult};

pub fn render(frame: &mut Frame, app: &App, qv: &QuestViewState, area: Rect) {
    if matches!(qv.test_result, Some(TestResult::Pass)) {
        render_victory_screen(frame, app, qv, area);
        return;
    }
    let quest = match app.get_quest(qv.quest_id) {
        Some(q) => q,
        None => return,
    };

    let done_badge = if app.completed.contains(&qv.quest_id) {
        " ✓"
    } else {
        ""
    };
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

    let cols = Layout::horizontal([
        Constraint::Percentage(qv.left_column_width),
        Constraint::Percentage(100_u16.saturating_sub(qv.left_column_width)),
    ])
    .split(inner);
    let left_col = cols[0];
    let right_col = cols[1];

    let inner_left_w = left_col.width.saturating_sub(2);
    let mut sol_height = 3;
    if qv.solutions_expanded {
        for (i, sol) in quest.solutions.iter().enumerate() {
            sol_height += 1;
            if qv.revealed_solutions.contains(&i) {
                sol_height += count_wrapped_lines(sol, inner_left_w);
            }
        }
    }
    let max_sol_height = inner.height.saturating_sub(5); // Ensure instructions get at least 5 lines
    sol_height = sol_height.min(max_sol_height).max(3);

    let left_chunks =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(sol_height)]).split(left_col);

    render_instructions(frame, qv, &quest.instructions, left_chunks[0]);
    render_solutions(frame, qv, &quest.solutions, left_chunks[1]);

    let mut right_constraints = vec![Constraint::Fill(1)]; // terminal
    if qv.has_answer_input {
        right_constraints.push(Constraint::Length(3)); // answer input
    }
    right_constraints.push(Constraint::Length(3)); // result
    right_constraints.push(Constraint::Length(3)); // buttons

    let right_chunks = Layout::vertical(right_constraints).split(right_col);

    let mut idx = 0;
    render_terminal(frame, qv, right_chunks[idx]);
    idx += 1;
    if qv.has_answer_input {
        let prompt = quest.submit_prompt.as_deref().unwrap_or("Your answer");
        render_answer_input(frame, qv, prompt, right_chunks[idx]);
        idx += 1;
    }
    render_result(frame, qv, right_chunks[idx]);
    idx += 1;
    render_buttons(frame, qv, right_chunks[idx]);
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

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

fn render_instructions(frame: &mut Frame, qv: &QuestViewState, instructions: &str, area: Rect) {
    let focused = qv.focus == QuestFocus::Instructions;
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let title = if focused {
        " Instructions (↑/↓ scroll  ←/→ resize  Tab next) "
    } else {
        " Instructions "
    };
    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(border_color));
    let text = instructions.to_string();
    let inner = block.inner(area);
    let total_lines = count_wrapped_lines(&text, inner.width);
    let max_scroll = total_lines.saturating_sub(inner.height);
    qv.max_instructions_scroll.set(max_scroll as usize);

    let scroll = qv.instructions_scroll_offset.min(max_scroll as usize) as u16;

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

fn render_solutions(frame: &mut Frame, qv: &QuestViewState, solutions: &[String], area: Rect) {
    let focused = qv.focus == QuestFocus::Solutions;
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let title = if focused {
        " Solutions (Enter to toggle, Arrows/Tab to navigate) "
    } else {
        " Solutions "
    };

    let block = Block::bordered()
        .title(title)
        .border_style(Style::new().fg(border_color));

    let mut lines = Vec::new();

    if qv.solutions_expanded {
        let mut tabs = vec![Span::raw(" ")];
        let num_sols = solutions.len();

        for (i, _) in solutions.iter().enumerate() {
            let is_selected = focused && qv.selected_solution_idx == i;
            let is_active = qv.revealed_solutions.contains(&i);

            let style = if is_selected {
                Style::new().bg(Color::White).fg(Color::Black).bold()
            } else if is_active {
                Style::new().fg(Color::Green).bold()
            } else {
                Style::new().fg(Color::Cyan)
            };

            tabs.push(Span::styled(format!(" [ {} ] ", i + 1), style));
            tabs.push(Span::raw(" "));
        }
        
        // Add an 'X' button to close
        let close_selected = focused && qv.selected_solution_idx == num_sols;
        let close_style = if close_selected {
            Style::new().bg(Color::White).fg(Color::Black).bold()
        } else {
            Style::new().fg(Color::Red)
        };
        tabs.push(Span::styled(" [ X ] ", close_style));

        lines.push(Line::from(tabs));
        lines.push(Line::from(Span::styled(
            "─".repeat(area.width.saturating_sub(2) as usize),
            Style::new().fg(Color::DarkGray),
        )));

        // Show active solution content
        let mut found = false;
        for (i, sol) in solutions.iter().enumerate() {
            if qv.revealed_solutions.contains(&i) {
                found = true;
                for line in sol.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::new().fg(Color::Gray),
                    )));
                }
            }
        }
        if !found {
            lines.push(Line::from(Span::styled(
                "  Select a part to view the solution",
                Style::new().fg(Color::DarkGray).italic(),
            )));
        }
    } else {
        let is_selected = focused && qv.selected_solution_idx == 0;
        let style = if is_selected {
            Style::new().bg(Color::White).fg(Color::Black).bold()
        } else {
            Style::new().fg(Color::Cyan)
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("▶ Show Solutions", style),
        ]));
    }

    frame.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_terminal(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let focused = qv.focus == QuestFocus::Terminal;
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let title = if focused {
        " Terminal (Enter to run  Tab navigate  Shift+↑↓↔ scroll  Ctrl+V paste  Ctrl+C copy output) "
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
        let style = if is_cursor {
            Style::new().fg(Color::Black).bg(Color::White)
        } else {
            Style::new()
        };

        if c == '\n' {
            if is_cursor {
                spans.push(Span::styled(
                    "█",
                    Style::new().fg(Color::Black).bg(Color::White),
                ));
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
            Style::new().bg(Color::White).fg(Color::Black),
        ));
    }
    lines.push(Line::from(spans));

    // Scroll so the bottom (live input) is always visible, accounting for user scroll_offset.
    let inner_w = inner.width.max(1) as usize;
    let total_lines: u16 = lines
        .iter()
        .map(|line| {
            let len = line.width();
            if len == 0 {
                1
            } else {
                ((len.saturating_sub(1) / inner_w) + 1) as u16
            }
        })
        .sum();

    let visible = inner.height;
    let max_scroll = total_lines.saturating_sub(visible);
    qv.max_terminal_scroll.set(max_scroll as usize);
    let effective_scroll_offset = qv.scroll_offset.min(max_scroll as usize) as u16;
    let scroll = max_scroll.saturating_sub(effective_scroll_offset);

    frame.render_widget(
        Paragraph::new(lines)
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

fn render_answer_input(frame: &mut Frame, qv: &QuestViewState, prompt: &str, area: Rect) {
    let focused = qv.focus == QuestFocus::Answer;
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };
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

    let sub_s = if qv.focus == QuestFocus::Submit {
        sel
    } else {
        normal
    };
    let back_s = if qv.focus == QuestFocus::Back {
        sel
    } else {
        normal
    };

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("[ Submit ]", sub_s),
        Span::raw("   "),
        Span::styled("[ Back ]", back_s),
        Span::raw("   "),
        Span::styled("Tab/←→ switch  Esc back", Style::new().fg(Color::DarkGray)),
    ]);

    let block = Block::bordered().border_style(Style::new().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn render_victory_screen(frame: &mut Frame, app: &App, _qv: &QuestViewState, area: Rect) {
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(15),
        Constraint::Fill(1),
    ])
    .split(area);

    let inner_chunks = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(60),
        Constraint::Fill(1),
    ])
    .split(chunks[1]);

    let victory_area = inner_chunks[1];

    // Better Firework animation logic
    let tick = app.tick;
    let frame_idx = (tick / 2) % 10;

    let firework = match frame_idx {
        0 => vec![
            "               ",
            "       |       ",
            "       |       ",
            "               ",
            "               ",
        ],

        1 => vec![
            "               ",
            "       ^       ",
            "      /|\\      ",
            "       |       ",
            "               ",
        ],

        2 => vec![
            "       .       ",
            "      \\|/      ",
            "    -- * --    ",
            "      /|\\      ",
            "       '       ",
        ],

        3 => vec![
            "    \\  |  /    ",
            "   . \\ | / .   ",
            " ---  ***  --- ",
            "   . / | \\ .   ",
            "    /  |  \\    ",
        ],

        4 => vec![
            " *   \\ | /   * ",
            "   *  ***  *   ",
            " --  *****  -- ",
            "   *  ***  *   ",
            " *   / | \\   * ",
        ],

        5 => vec![
            " .   *   *   . ",
            "   \\  | |  /   ",
            " * --  *  -- * ",
            "   /  | |  \\   ",
            " .   *   *   . ",
        ],

        6 => vec![
            "   .   .   .   ",
            " .    * *    . ",
            "      . .      ",
            " .    * *    . ",
            "   .   .   .   ",
        ],

        7 => vec![
            "   .       .   ",
            "      . .      ",
            "               ",
            "      . .      ",
            "   .       .   ",
        ],

        _ => vec![
            "               ",
            "               ",
            "               ",
            "               ",
            "               ",
        ],
    };

    let block = Block::bordered()
        .border_style(Style::new().fg(Color::Green))
        .style(Style::new().bg(Color::Reset));

    let mut text = vec![Line::from("")];

    // Add animated fireworks at the top with varying colors
    let fw_color = match frame_idx {
        0..=2 => Color::White,
        3..=5 => Color::Yellow,
        6..=7 => Color::DarkGray,
        _ => Color::Reset,
    };

    for line in firework {
        text.push(
            Line::from(Span::styled(line, Style::new().fg(fw_color).bold()))
                .alignment(Alignment::Center),
        );
    }

    text.extend(vec![
        Line::from(""),
        Line::from(Span::styled(
            "COMPLETED QUEST",
            Style::new().fg(Color::Green).bold(),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![Span::raw(
            "Congratulations! You have completed this quest.",
        )])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![Span::styled(
            " [ Go Back ] ",
            Style::new().bg(Color::Yellow).fg(Color::Black).bold(),
        )])
        .alignment(Alignment::Center),
        Line::from(vec![Span::styled(
            "(press enter to go back to quests page)",
            Style::new().fg(Color::DarkGray),
        )])
        .alignment(Alignment::Center),
    ]);

    frame.render_widget(Paragraph::new(text).block(block), victory_area);
}
