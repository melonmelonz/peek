//! Persistent layout chrome: title bar, stat bars, footer dialogue line,
//! key hint bar.

use crate::app::App;
use crate::theme::Theme;
use peek_core::Mood;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Paragraph};
use ratatui::Frame;

pub fn split_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // title bar
            Constraint::Length(3), // stat bars
            Constraint::Min(8),    // body
            Constraint::Length(3), // dialogue + keys
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2], chunks[3])
}

pub fn render_title(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let creature = app.creature();
    let (name, stage, mood) = match creature {
        Some(c) => (c.true_name.clone(), c.stage.name().to_string(), mood_label(c.mood).to_string()),
        None => ("...".into(), "void".into(), "anxious".into()),
    };
    let title = Line::from(vec![
        Span::styled("PEEK ", Style::default().fg(theme.accent_pink).add_modifier(Modifier::BOLD)),
        Span::styled("examines embedded kernels   ", Style::default().fg(theme.dim)),
        Span::styled(format!("[ {name} ]"), Style::default().fg(theme.accent_cyan)),
        Span::raw("   "),
        Span::styled(format!("stage: {stage}"), Style::default().fg(theme.accent_violet)),
        Span::raw("   "),
        Span::styled(format!("mood: {mood}"), Style::default().fg(theme.accent_mint)),
    ]);
    let p = Paragraph::new(title).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(theme.accent_violet)),
    );
    frame.render_widget(p, area);
}

pub fn render_stats(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let s = app.stats();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(34), Constraint::Percentage(33), Constraint::Percentage(33)])
        .split(area);
    let bar = |label: &str, ratio: f32, color| {
        Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.dim))
                    .title(Line::from(Span::styled(
                        label.to_string(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ))),
            )
            .gauge_style(Style::default().fg(color).bg(theme.bg))
            .ratio(ratio.clamp(0.0, 1.0) as f64)
    };
    frame.render_widget(bar("nourishment", s.nourishment, theme.accent_pink), chunks[0]);
    frame.render_widget(bar("tether", s.tether, theme.accent_cyan), chunks[1]);
    frame.render_widget(bar("lucidity", s.lucidity, theme.accent_mint), chunks[2]);
}

pub fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme, app: &App, key_hint: &str) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);
    let line = app.current_dialogue.clone().unwrap_or_default();
    let dialogue = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.accent_violet)),
        Span::styled(line, Style::default().fg(theme.fg)),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.accent_violet)),
    );
    frame.render_widget(dialogue, chunks[0]);
    let keys = Paragraph::new(Line::from(vec![Span::styled(
        key_hint.to_string(),
        Style::default().fg(theme.dim),
    )]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(theme.accent_violet)),
    );
    frame.render_widget(keys, chunks[1]);
}

fn mood_label(m: Mood) -> &'static str {
    match m {
        Mood::Anxious => "anxious",
        Mood::Lucid => "lucid",
        Mood::Drifting => "drifting",
        Mood::Ravenous => "ravenous",
        Mood::Reverent => "reverent",
    }
}
