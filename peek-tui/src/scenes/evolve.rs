//! Evolve scene: stage transformation ceremony.
//!
//! Plays when `Creature::tick` (or a care action) reports that the
//! creature crossed a stage boundary. Three beats:
//!   1. The creature's silhouette pulses; old form fades.
//!   2. A vortex of unicode glyphs swirls in.
//!   3. The new form lands and is named.
//!
//! Pressing any key ends the scene early. The creature itself was
//! advanced before this scene was constructed; the scene is purely
//! presentation.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::input::{InputEvent, Key};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::sprite::SpriteWidget;
use crate::theme::Theme;
use peek_content::SpriteSet;
use peek_core::Stage;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use ratatui::Frame;

const FRAMES_FADE: u32 = 12;
const FRAMES_VORTEX: u32 = 24;
const FRAMES_REVEAL: u32 = 14;
const FRAMES_HOLD: u32 = 18;
const FRAMES_END: u32 = FRAMES_FADE + FRAMES_VORTEX + FRAMES_REVEAL + FRAMES_HOLD;

const VORTEX_GLYPHS: [char; 12] = ['·', '∘', '○', '◌', '◍', '◎', '●', '✦', '✧', '⋆', '∗', '✺'];

pub struct EvolveScene {
    pub theme: Theme,
    pub from_stage: Stage,
    pub to_stage: Stage,
    progress: u32,
}

impl EvolveScene {
    pub fn new(theme: Theme, from_stage: Stage, to_stage: Stage) -> Self {
        Self {
            theme,
            from_stage,
            to_stage,
            progress: 0,
        }
    }

    fn flavor(&self) -> &'static str {
        match self.to_stage {
            Stage::Sprout => "the shell parts. something looks back.",
            Stage::Knot => "it folds itself into a tighter shape. denser. surer.",
            Stage::Mawling => "the maw uncloses. it has questions of its own now.",
            Stage::Conduit => "channels open. it conducts a thought without flinching.",
            Stage::Cogent => "it is articulate. it is whole. it knows what it is.",
            Stage::Egg => "—",
        }
    }
}

impl Scene for EvolveScene {
    fn id(&self) -> SceneId {
        SceneId::Evolve
    }

    fn handle(&mut self, ev: &InputEvent, _app: &mut App) -> SceneAction {
        let InputEvent::Key(k) = ev;
        match k {
            Key::Char('Q') => SceneAction::Quit,
            // Any key fast-forwards to the end of the ceremony.
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
            return SceneAction::Goto(SceneId::Idle);
        }
        SceneAction::Stay
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, stats, body, footer) = split_layout(area);
        render_title(frame, title, &self.theme, app);
        render_stats(frame, stats, &self.theme, app);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(self.theme.accent_violet))
            .title(Span::styled(
                " transformation ",
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);

        // Phase derived from progress.
        let p = self.progress;
        let phase_fade = p < FRAMES_FADE;
        let phase_vortex = !phase_fade && p < FRAMES_FADE + FRAMES_VORTEX;
        let phase_reveal =
            !phase_fade && !phase_vortex && p < FRAMES_FADE + FRAMES_VORTEX + FRAMES_REVEAL;

        // Sprite area on the left half, dialogue on the right half.
        let split_x = inner.x + inner.width / 2;
        let left = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width / 2,
            height: inner.height,
        };
        let right = Rect {
            x: split_x,
            y: inner.y,
            width: inner.width - inner.width / 2,
            height: inner.height,
        };

        // Left: sprite that morphs from old → new.
        if let Some(c) = app.creature() {
            let frame_bit = (app.frame_idx / 6) % 2;
            let stage_to_show = if phase_fade || phase_vortex {
                self.from_stage
            } else {
                self.to_stage
            };
            let sprite_text = SpriteSet::frame(stage_to_show, c.mood, frame_bit)
                .unwrap_or_else(|| "(missing sprite)".to_string());
            let widget = SpriteWidget {
                stage: stage_to_show,
                mood: c.mood,
                frame: frame_bit,
                mutations: &c.mutations,
                sprite_text,
                theme: self.theme,
                frame_idx: app.frame_idx,
            };
            frame.render_widget(widget, left);
        }

        // Right: phase-dependent narration.
        let mut lines: Vec<Line> = Vec::new();
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("{} → {}", self.from_stage.name(), self.to_stage.name()),
            Style::default()
                .fg(self.theme.accent_cyan)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        if phase_fade {
            lines.push(Line::from(Span::styled(
                "the old shape softens.",
                Style::default()
                    .fg(self.theme.dim)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else if phase_vortex {
            // Render two rows of swirling glyphs that shuffle each tick.
            let row1: String = (0..12)
                .map(|i| {
                    let idx = (app.frame_idx as usize + i * 3) % VORTEX_GLYPHS.len();
                    VORTEX_GLYPHS[idx]
                })
                .collect();
            let row2: String = (0..12)
                .map(|i| {
                    let idx = (app.frame_idx as usize + i * 5 + 7) % VORTEX_GLYPHS.len();
                    VORTEX_GLYPHS[idx]
                })
                .collect();
            lines.push(Line::from(Span::styled(
                row1,
                Style::default().fg(self.theme.accent_violet),
            )));
            lines.push(Line::from(Span::styled(
                row2,
                Style::default().fg(self.theme.accent_pink),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "something is rearranging.",
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::ITALIC),
            )));
        } else if phase_reveal {
            lines.push(Line::from(Span::styled(
                self.flavor(),
                Style::default().fg(self.theme.fg),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                self.flavor(),
                Style::default().fg(self.theme.fg),
            )));
            lines.push(Line::from(""));
            // Soft blink prompt during the hold.
            let modifier = if (app.frame_idx / 4) % 2 == 0 {
                Modifier::BOLD
            } else {
                Modifier::DIM
            };
            lines.push(Line::from(Span::styled(
                "press any key to continue.",
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(modifier),
            )));
        }

        let para = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        frame.render_widget(para, right);

        render_footer(frame, footer, &self.theme, app, "any key skip   Q quit");
    }
}
