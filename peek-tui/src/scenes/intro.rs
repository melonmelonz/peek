//! Intro scene: neon title card with starfield + reveal animation.
//!
//! Played once on first launch and replayable from the demo menu. Each
//! beat lands at a known frame so the cadence is the same on every run.
//! Pressing any key skips to the next beat; pressing past the last beat
//! transitions to the hatch sequence.

use crate::app::App;
use crate::chrome::{render_footer, split_layout};
use crate::input::{InputEvent, Key};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::theme::Theme;
use peek_core::Stage;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

/// Frames-per-tick map. The runtime ticks at ~120ms, so 60 frames ≈ 7s.
const FRAMES_TITLE_IN: u32 = 8;
const FRAMES_SUB_IN: u32 = 16;
const FRAMES_LINE1: u32 = 28;
const FRAMES_LINE2: u32 = 40;
const FRAMES_LINE3: u32 = 52;
const FRAMES_PROMPT: u32 = 60;
const FRAMES_END: u32 = 80;

const TITLE: [&str; 6] = [
    r#" ____  _____ _____ _  __ "#,
    r#"|  _ \| ____| ____| |/ / "#,
    r#"| |_) |  _| |  _| | ' /  "#,
    r#"|  __/| |___| |___| . \  "#,
    r#"|_|   |_____|_____|_|\_\ "#,
    r#"                         "#,
];

const SUBTITLE: &str = "examines embedded kernels";

const LINE_1: &str = "you are about to keep a small thing alive.";
const LINE_2: &str = "feed it questions. tend its tether. read with it.";
const LINE_3: &str = "it will grow. it will mutate. one day it will go.";

const PROMPT: &str = "press any key to begin.";

/// 32 deterministic star anchor points laid out across an 80x18 area.
/// Each entry is (x, y, twinkle phase). The render code modulates the
/// star glyph by `(frame_idx + phase) % cycle` to get a varied twinkle.
const STARS: &[(u16, u16, u8)] = &[
    (3, 1, 0),
    (10, 2, 2),
    (18, 1, 5),
    (26, 3, 1),
    (34, 1, 7),
    (42, 2, 3),
    (50, 1, 6),
    (58, 3, 4),
    (66, 2, 0),
    (74, 1, 5),
    (5, 5, 1),
    (15, 6, 3),
    (25, 4, 6),
    (35, 5, 2),
    (45, 6, 0),
    (55, 4, 4),
    (65, 5, 7),
    (75, 6, 3),
    (2, 9, 5),
    (12, 10, 1),
    (22, 11, 3),
    (32, 9, 6),
    (42, 10, 0),
    (52, 11, 4),
    (62, 9, 2),
    (72, 10, 7),
    (8, 14, 4),
    (20, 15, 0),
    (32, 14, 6),
    (44, 15, 2),
    (56, 14, 5),
    (68, 15, 1),
];

const STAR_GLYPHS: [char; 4] = ['.', '+', '*', '·'];

pub struct IntroScene {
    pub theme: Theme,
    progress: u32,
}

impl IntroScene {
    pub fn new(theme: Theme) -> Self {
        Self { theme, progress: 0 }
    }
}

impl Scene for IntroScene {
    fn id(&self) -> SceneId {
        SceneId::Intro
    }

    fn handle(&mut self, ev: &InputEvent, _app: &mut App) -> SceneAction {
        let InputEvent::Key(k) = ev;
        match k {
            Key::Char('Q') => SceneAction::Quit,
            // Any key advances to the end of the intro; tick will then
            // hand off to the hatch sequence on the next frame.
            _ => {
                self.progress = FRAMES_END;
                SceneAction::Stay
            }
        }
    }

    fn tick(&mut self, app: &mut App) -> SceneAction {
        app.frame_idx = app.frame_idx.wrapping_add(1);
        self.progress = self.progress.saturating_add(1);
        if self.progress >= FRAMES_END {
            // First-launch path: a fresh egg exists, so play Hatch next.
            // Replaying the intro from Idle skips the hatch animation and
            // returns straight to Idle — no surprise stage changes.
            let going_to_hatch = app
                .creature()
                .map(|c| c.stage == Stage::Egg)
                .unwrap_or(true);
            return SceneAction::Goto(if going_to_hatch {
                SceneId::Hatch
            } else {
                SceneId::Idle
            });
        }
        SceneAction::Stay
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, _stats, body, footer) = split_layout(area);

        // Reuse the title bar so the chrome stays consistent — we draw a
        // neon double-bordered block over the body and lay the intro on top.
        let title_block = Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(self.theme.accent_violet))
            .title(Span::styled(
                " welcome ",
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::BOLD),
            ));
        frame.render_widget(title_block, title);

        let body_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(self.theme.accent_violet));
        let inner = body_block.inner(body);
        frame.render_widget(body_block, body);

        // Twinkling starfield.
        for (sx, sy, phase) in STARS {
            if *sx >= inner.width || *sy >= inner.height {
                continue;
            }
            let bucket = ((app.frame_idx as u32 + *phase as u32) / 2) as usize % STAR_GLYPHS.len();
            let glyph = STAR_GLYPHS[bucket];
            let cx = inner.x + *sx;
            let cy = inner.y + *sy;
            // The star color cycles softly between dim and cyan to suggest
            // the cosmic background you'd expect from an eldritch field.
            let color = if bucket >= 2 {
                self.theme.accent_cyan
            } else {
                self.theme.dim
            };
            let cell = &mut frame.buffer_mut()[(cx, cy)];
            cell.set_char(glyph);
            cell.set_style(Style::default().fg(color));
        }

        // Title (PEEK), centered.
        let title_y_start = inner.y + 2;
        let revealed = self.progress.min(FRAMES_TITLE_IN);
        let reveal_ratio = revealed as f32 / FRAMES_TITLE_IN as f32;
        let title_w = TITLE[0].chars().count() as u16;
        let title_left = inner.x + inner.width.saturating_sub(title_w) / 2;
        for (row_idx, row) in TITLE.iter().enumerate() {
            let y = title_y_start + row_idx as u16;
            if y >= inner.y + inner.height {
                break;
            }
            for (col_idx, ch) in row.chars().enumerate() {
                let x = title_left + col_idx as u16;
                if x >= inner.x + inner.width {
                    break;
                }
                if ch == ' ' {
                    continue;
                }
                // Reveal left-to-right.
                let progress_at = col_idx as f32 / title_w.max(1) as f32;
                if progress_at > reveal_ratio {
                    continue;
                }
                let style = if (app.frame_idx as usize + col_idx) % 11 == 0 {
                    Style::default()
                        .fg(self.theme.accent_pink)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(self.theme.accent_violet)
                        .add_modifier(Modifier::BOLD)
                };
                let cell = &mut frame.buffer_mut()[(x, y)];
                cell.set_char(ch);
                cell.set_style(style);
            }
        }

        // Subtitle, story lines, and prompt are drawn as a paragraph.
        let mut lines: Vec<Line> = Vec::new();
        for _ in 0..(TITLE.len() as u16 + 3) {
            lines.push(Line::from(""));
        }
        if self.progress >= FRAMES_SUB_IN {
            lines.push(Line::from(Span::styled(
                SUBTITLE.to_string(),
                Style::default()
                    .fg(self.theme.accent_cyan)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(""));
        push_reveal(
            &mut lines,
            LINE_1,
            self.progress,
            FRAMES_LINE1,
            self.theme.fg,
        );
        push_reveal(
            &mut lines,
            LINE_2,
            self.progress,
            FRAMES_LINE2,
            self.theme.fg,
        );
        push_reveal(
            &mut lines,
            LINE_3,
            self.progress,
            FRAMES_LINE3,
            self.theme.fg,
        );
        lines.push(Line::from(""));
        if self.progress >= FRAMES_PROMPT {
            // Soft blink on the prompt by toggling DIM/BOLD with frame_idx.
            let modifier = if (app.frame_idx / 4) % 2 == 0 {
                Modifier::BOLD
            } else {
                Modifier::DIM
            };
            lines.push(Line::from(Span::styled(
                PROMPT.to_string(),
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(modifier),
            )));
        }

        let p = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(p, inner);

        render_footer(frame, footer, &self.theme, app, "any key skip   Q quit");
    }
}

/// Push either a typewriter-revealed slice of `text` or a blank, depending
/// on whether `progress` has reached `at`. Reveal lasts 8 frames.
fn push_reveal(
    lines: &mut Vec<Line<'static>>,
    text: &'static str,
    progress: u32,
    at: u32,
    color: ratatui::style::Color,
) {
    if progress < at {
        lines.push(Line::from(""));
        return;
    }
    let elapsed = progress - at;
    let total = text.chars().count() as u32;
    let visible = elapsed.saturating_mul(3).min(total);
    let s: String = text.chars().take(visible as usize).collect();
    lines.push(Line::from(Span::styled(s, Style::default().fg(color))));
}
