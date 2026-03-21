use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::app::{App, QuestFocus, QuestViewState, TestResult};
use crate::quests;

pub fn render(frame: &mut Frame, app: &App, qv: &QuestViewState, area: Rect) {
    let quest = match quests::get(qv.quest_id) {
        Some(q) => q,
        None => return,
    };

    let done_badge = if app.completed.contains(&qv.quest_id) {
        " ✓"
    } else {
        ""
    };
    let title = format!(" Quest {}/{}: {}{} ", qv.quest_id, 24, quest.title, done_badge);

    let outer = Block::bordered()
        .title(title.as_str())
        .border_style(Style::new().fg(Color::Cyan));
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Split inner area: instructions | input | output | result | buttons
    let chunks = Layout::vertical([
        Constraint::Percentage(30), // instructions
        Constraint::Length(3),      // command input
        Constraint::Fill(1),        // output
        Constraint::Length(3),      // test result
        Constraint::Length(3),      // buttons
    ])
    .split(inner);

    render_instructions(frame, quest.instructions, quest.hint, chunks[0]);
    render_input(frame, qv, chunks[1]);
    render_output(frame, qv, chunks[2]);
    render_result(frame, qv, chunks[3]);
    render_buttons(frame, qv, chunks[4]);
}

fn render_instructions(frame: &mut Frame, instructions: &str, hint: &str, area: Rect) {
    let block = Block::bordered()
        .title(" Instructions ")
        .border_style(Style::new().fg(Color::DarkGray));

    let text = format!("{}\n\nHint: {}", instructions, hint);
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: false }),
        area,
    );
}

fn render_input(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let focused = qv.focus == QuestFocus::Input;
    let border_color = if focused { Color::Yellow } else { Color::DarkGray };

    let block = Block::bordered()
        .title(" Command (Enter to run) ")
        .border_style(Style::new().fg(border_color));

    // Show a block cursor inside the input text
    let before = &qv.input[..qv.cursor];
    let at = if focused { "█" } else { "" };
    let after = &qv.input[qv.cursor..];

    frame.render_widget(
        Paragraph::new(format!("{}{}{}", before, at, after)).block(block),
        area,
    );
}

fn render_output(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let block = Block::bordered()
        .title(" Output ")
        .border_style(Style::new().fg(Color::DarkGray));

    let text = if qv.output.is_empty() {
        "Run your command above to see output here...".to_string()
    } else {
        qv.output.clone()
    };

    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn render_result(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let (text, color) = match &qv.test_result {
        Some(TestResult::Pass) => ("  ✓  Quest complete! Well done.", Color::Green),
        Some(TestResult::Fail(msg)) => (msg.as_str(), Color::Red),
        None => ("  Submit your output to check the answer.", Color::DarkGray),
    };

    let block = Block::bordered().border_style(Style::new().fg(color));
    frame.render_widget(
        Paragraph::new(text).style(Style::new().fg(color).bold()).block(block),
        area,
    );
}

fn render_buttons(frame: &mut Frame, qv: &QuestViewState, area: Rect) {
    let sel = Style::new().bg(Color::Yellow).fg(Color::Black).bold();
    let normal = Style::new().fg(Color::White);

    let run_s = if qv.focus == QuestFocus::Run { sel } else { normal };
    let sub_s = if qv.focus == QuestFocus::Submit { sel } else { normal };
    let back_s = if qv.focus == QuestFocus::Back { sel } else { normal };

    let line = Line::from(vec![
        Span::raw("  "),
        Span::styled("[ Run ]", run_s),
        Span::raw("   "),
        Span::styled("[ Submit ]", sub_s),
        Span::raw("   "),
        Span::styled("[ Back ]", back_s),
        Span::raw("   "),
        Span::styled("Tab/←→ switch  Esc back", Style::new().fg(Color::DarkGray)),
    ]);

    let block = Block::bordered().border_style(Style::new().fg(Color::DarkGray));
    frame.render_widget(Paragraph::new(line).block(block), area);
}
