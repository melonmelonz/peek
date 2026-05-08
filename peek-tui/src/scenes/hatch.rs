//! Hatching animation: 20 frames of egg morphing into sprout, with 3
//! dialogue beats. Transitions to Idle when finished.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::sprite::SpriteWidget;
use crate::theme::Theme;
use crossterm::event::{Event, KeyCode};
use peek_content::SpriteSet;
use peek_core::Stage;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;

const TOTAL_FRAMES: u32 = 20;

pub struct HatchScene {
    pub theme: Theme,
    progress: u32,
    spoken: u8,
}

impl HatchScene {
    pub fn new(theme: Theme) -> Self {
        Self { theme, progress: 0, spoken: 0 }
    }
}

impl Scene for HatchScene {
    fn id(&self) -> SceneId {
        SceneId::Hatch
    }

    fn handle(&mut self, ev: &Event, _app: &mut App) -> SceneAction {
        if let Event::Key(k) = ev {
            if matches!(k.code, KeyCode::Char('Q') | KeyCode::Char('q')) {
                return SceneAction::Quit;
            }
            if matches!(k.code, KeyCode::Enter | KeyCode::Char(' ')) {
                self.progress = TOTAL_FRAMES; // skip the animation
            }
        }
        SceneAction::Stay
    }

    fn tick(&mut self, app: &mut App) -> SceneAction {
        self.progress = self.progress.saturating_add(1);
        let beats = [
            (5, "hatch"),
            (12, "hatch"),
            (18, "hatch"),
        ];
        for (frame_at, ev) in beats {
            if self.progress == frame_at && self.spoken < 3 {
                app.say(ev);
                self.spoken += 1;
            }
        }
        if self.progress >= TOTAL_FRAMES + 6 {
            // After hatching, advance the creature out of Egg into Sprout
            // so the player has something to look at.
            if let Some(c) = app.creature_mut() {
                if c.stage == Stage::Egg {
                    c.advance_stage(chrono::Utc::now());
                }
            }
            let _ = app.save();
            return SceneAction::Goto(SceneId::Idle);
        }
        SceneAction::Stay
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, stats, body, footer) = split_layout(area);
        render_title(frame, title, &self.theme, app);
        render_stats(frame, stats, &self.theme, app);

        let stage = if self.progress < TOTAL_FRAMES / 2 {
            Stage::Egg
        } else {
            Stage::Sprout
        };
        let mood = peek_core::Mood::Anxious;
        let frame_bit = ((self.progress / 2) % 2) as u8;
        let sprite_text = SpriteSet::frame(stage, mood, frame_bit)
            .unwrap_or_else(|| "(missing sprite)".to_string());

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent_violet))
            .title(Span::styled(
                " hatching ",
                Style::default().fg(self.theme.accent_pink).add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);
        let widget = SpriteWidget {
            stage,
            mood,
            frame: frame_bit,
            mutations: &[],
            sprite_text,
            theme: self.theme,
        };
        frame.render_widget(widget, inner);

        render_footer(frame, footer, &self.theme, app, "space skip   Q quit");
    }
}
