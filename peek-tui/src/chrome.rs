//! Persistent layout chrome: title bar, stat bars, footer dialogue line,
//! key hint bar.
//!
//! Hand-rendered to keep the neon-90s aesthetic: chunky block-character
//! stat bars with gradient glow, ASCII title art with rotating accent
//! colors, and double-rule borders on the dialogue footer.

use crate::app::App;
use crate::theme::Theme;
use peek_core::{Mood, Stats};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Widget};
use ratatui::Frame;

pub fn split_layout(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // title bar (4 lines of art + 1 spacer)
            Constraint::Length(4), // stat bars
            Constraint::Min(8),    // body
            Constraint::Length(4), // dialogue + keys
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2], chunks[3])
}

const TITLE_LINES: [&str; 4] = [
    "  ___  ___ ___ _  __     examines",
    " | _ \\| __| __| |/ /     embedded",
    " |  _/| _|| _|| ' <      kernels",
    " |_|  |___|___|_|\\_\\     ",
];

pub fn render_title(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let creature = app.creature();
    let (name, stage, mood) = match creature {
        Some(c) => (
            c.true_name.clone(),
            c.stage.name().to_string(),
            mood_label(c.mood).to_string(),
        ),
        None => ("...".into(), "void".into(), "anxious".into()),
    };
    // Cycle accent color based on the global frame index for a neon shimmer.
    let cycle = [
        theme.accent_pink,
        theme.accent_cyan,
        theme.accent_violet,
        theme.accent_mint,
    ];
    let phase = ((app.frame_idx as usize) / 6) % cycle.len();
    let primary = cycle[phase];
    let secondary = cycle[(phase + 2) % cycle.len()];

    let mut lines: Vec<Line> = Vec::new();
    for (i, raw) in TITLE_LINES.iter().enumerate() {
        let color = if i == 0 || i == 3 { secondary } else { primary };
        // Split: ascii letters, then narrative tail.
        if let Some((art, tail)) = raw.split_once("    ") {
            lines.push(Line::from(vec![
                Span::styled(
                    art.to_string(),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("    {tail}"),
                    Style::default()
                        .fg(theme.dim)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        } else {
            lines.push(Line::from(Span::styled(
                raw.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
        }
    }
    // 5th-line status strip
    let status = Line::from(vec![
        Span::styled("  [ ", Style::default().fg(theme.dim)),
        Span::styled(
            name,
            Style::default()
                .fg(theme.accent_cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ]   stage: ", Style::default().fg(theme.dim)),
        Span::styled(stage, Style::default().fg(theme.accent_violet)),
        Span::styled("   mood: ", Style::default().fg(theme.dim)),
        Span::styled(mood, Style::default().fg(theme.accent_mint)),
    ]);

    // Render title art lines + status, using the upper area.
    let p = Paragraph::new({
        let mut all = lines;
        all.push(status);
        all
    });
    frame.render_widget(p, area);
}

pub fn render_stats(frame: &mut Frame, area: Rect, theme: &Theme, app: &App) {
    let s = app.stats();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(area);
    let bars = [
        ("nourishment", s.nourishment, theme.accent_pink),
        ("tether", s.tether, theme.accent_cyan),
        ("lucidity", s.lucidity, theme.accent_mint),
    ];
    for (i, (label, ratio, color)) in bars.iter().enumerate() {
        let widget = NeonBar {
            label: label.to_string(),
            ratio: *ratio,
            color: *color,
            theme: *theme,
            frame_idx: app.frame_idx,
            stats: &s,
        };
        frame.render_widget(widget, chunks[i]);
    }
}

pub fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme, app: &App, key_hint: &str) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);
    let line = app.current_dialogue.clone().unwrap_or_default();
    // Ghost-double-quote opener for the eldritch register.
    let prefix_style = Style::default()
        .fg(theme.accent_violet)
        .add_modifier(Modifier::BOLD);
    let dialogue = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" >> ", prefix_style),
            Span::styled(
                line,
                Style::default().fg(theme.fg).add_modifier(Modifier::ITALIC),
            ),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(theme.accent_violet)),
    );
    frame.render_widget(dialogue, chunks[0]);
    let keys = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(" {key_hint} "),
            Style::default().fg(theme.dim),
        )),
    ])
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_type(BorderType::Double)
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

/// Custom stat bar: neon block characters with a glow trail and pulsing
/// cap when the stat is dangerously low.
struct NeonBar<'a> {
    label: String,
    ratio: f32,
    color: Color,
    theme: Theme,
    frame_idx: u8,
    stats: &'a Stats,
}

impl Widget for NeonBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 6 || area.height < 3 {
            return;
        }
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        let _ = self.stats; // reserved for future per-stat reactivity

        // Border frame in the bar's accent color, dimmer when ratio is low.
        let border_color = if self.ratio < 0.25 {
            self.theme.warn
        } else {
            self.theme.dim
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(
                format!(" {} ", self.label),
                Style::default().fg(self.color).add_modifier(Modifier::BOLD),
            ));
        block.render(area, buf);

        // Bar body: render with eighths block characters for sub-cell precision.
        let total_cells = inner.width as f32;
        let filled_f = (self.ratio.clamp(0.0, 1.0) * total_cells).max(0.0);
        let full_cells = filled_f.floor() as u16;
        let remainder = filled_f - full_cells as f32;
        // Map remainder to one of: ' '  '▏'  '▎'  '▍'  '▌'  '▋'  '▊'  '▉'  '█'
        const PARTIAL: &[char] = &[' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
        let partial_char = PARTIAL[(remainder * 8.0).floor() as usize % PARTIAL.len()];

        // Shimmer: brighten one column near the leading edge.
        let shimmer_at = full_cells.saturating_sub((self.frame_idx % 4) as u16);

        let row = inner.y + inner.height / 2;
        for i in 0..inner.width {
            let x = inner.x + i;
            if x >= inner.x + inner.width || row >= inner.y + inner.height {
                break;
            }
            let cell = &mut buf[(x, row)];
            if i < full_cells {
                let style = if i == shimmer_at {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(self.color).add_modifier(Modifier::BOLD)
                };
                cell.set_char('█');
                cell.set_style(style);
            } else if i == full_cells && remainder > 0.0 {
                cell.set_char(partial_char);
                cell.set_style(Style::default().fg(self.color));
            } else {
                cell.set_char('░');
                cell.set_style(Style::default().fg(self.theme.dim));
            }
        }

        // Numeric readout above the bar, right-aligned.
        let pct = format!(
            "{:>3}%",
            (self.ratio.clamp(0.0, 1.0) * 100.0).round() as u16
        );
        let label_y = inner.y;
        let chars: Vec<char> = pct.chars().collect();
        let total_w = chars.len() as u16;
        let start_x = inner.x + inner.width.saturating_sub(total_w);
        for (j, c) in chars.iter().enumerate() {
            let x = start_x + j as u16;
            if x >= inner.x + inner.width {
                break;
            }
            if label_y >= inner.y + inner.height {
                break;
            }
            let cell = &mut buf[(x, label_y)];
            cell.set_char(*c);
            cell.set_style(Style::default().fg(self.color).add_modifier(Modifier::DIM));
        }
    }
}
