//! Hatching animation: a multi-stage neon ritual.
//!
//! Phase 1: a still egg pulses against the starfield.
//! Phase 2: cracks spider across its surface.
//! Phase 3: the egg breaks open and the sprout emerges.
//! Three dialogue beats land at frames 6, 14, 24. After the morph, the
//! creature is advanced from Egg to Sprout and we transition to Idle.

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
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

const FRAMES_EGG_PULSE: u32 = 12;
const FRAMES_CRACKING: u32 = 14;
const FRAMES_REVEAL: u32 = 8;
const FRAMES_TOTAL: u32 = FRAMES_EGG_PULSE + FRAMES_CRACKING + FRAMES_REVEAL;

const EGG_BASE: &str = r#"
        .   *   .   *
     *    .--""--.    *
   .    .'  ____  '.   .
        /  .'    '.  \
       ;  / o    o \  ;
       |  | .    . |  |
       ;  \  vvvv  /  ;
        \  '.____.'  /
       .  '.        .'
     *    '.------.'    *
       .    \.--./    .
        *  .'    '. *
"#;

const EGG_CRACK_1: &str = r#"
        .   *   .   *
     *    .--""--.    *
   .    .'  ____  '.   .
        /  .'/   '.  \
       ;  / o /   o \  ;
       |  | ./    . |  |
       ;  \ /vvvv  /  ;
        \  '.____.'  /
       .  '.        .'
     *    '.------.'    *
       .    \.--./    .
        *  .'    '. *
"#;

const EGG_CRACK_2: &str = r#"
        .   *   .   *
     *    .--""--.    *
   .    .'/ ____ \'.   .
        /\/.'    '.\/\
       ;  / O    O \  ;
       |  |\\....//|  |
       ;  \  vvvv  /  ;
        \//'.____.'\\/
       .  '. /  \  .'
     *    '.------.'    *
       .    \.--./    .
        *  .'    '. *
"#;

const EGG_BREAKING: &str = r#"
   *  *   .  *   *  .   *
       \\  '----'  //
   *    \\.----..// *
       . `\.    ./` .
        ;  | OO |  ;
   *   |   /\v/\   |   *
       ;  /------\  ;
        \//      \\/
   .   //    *   \\   .
      //___ * * ___\\
   * '''      '''  *
       *           *
"#;

pub struct HatchScene {
    pub theme: Theme,
    progress: u32,
    spoken: u8,
}

impl HatchScene {
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            progress: 0,
            spoken: 0,
        }
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
                self.progress = FRAMES_TOTAL;
            }
        }
        SceneAction::Stay
    }

    fn tick(&mut self, app: &mut App) -> SceneAction {
        self.progress = self.progress.saturating_add(1);
        let beats = [(6, "hatch"), (14, "hatch"), (24, "hatch")];
        for (frame_at, ev) in beats {
            if self.progress == frame_at && self.spoken < 3 {
                app.say(ev);
                self.spoken += 1;
            }
        }
        if self.progress >= FRAMES_TOTAL + 4 {
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

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(self.theme.accent_violet))
            .title(Span::styled(
                " hatching ",
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);

        // Pick which sprite to show based on the phase.
        let p = self.progress;
        let sprite_text: String = if p < FRAMES_EGG_PULSE {
            // Pulse: alternate between two egg frames.
            if (p / 3) % 2 == 0 {
                EGG_BASE.trim_start_matches('\n').to_string()
            } else {
                SpriteSet::frame(Stage::Egg, peek_core::Mood::Anxious, 1)
                    .unwrap_or_else(|| EGG_BASE.into())
            }
        } else if p < FRAMES_EGG_PULSE + FRAMES_CRACKING {
            let inside = (p - FRAMES_EGG_PULSE) as f32 / FRAMES_CRACKING as f32;
            if inside < 0.5 {
                EGG_CRACK_1.trim_start_matches('\n').to_string()
            } else {
                EGG_CRACK_2.trim_start_matches('\n').to_string()
            }
        } else if p < FRAMES_TOTAL {
            EGG_BREAKING.trim_start_matches('\n').to_string()
        } else {
            // Final reveal: a sprout emerging.
            SpriteSet::frame(Stage::Sprout, peek_core::Mood::Anxious, ((p / 2) % 2) as u8)
                .unwrap_or_else(|| EGG_BREAKING.into())
        };

        let stage = if p < FRAMES_TOTAL {
            Stage::Egg
        } else {
            Stage::Sprout
        };

        let widget = SpriteWidget {
            stage,
            mood: peek_core::Mood::Anxious,
            frame: ((p / 2) % 2) as u8,
            mutations: &[],
            sprite_text,
            theme: self.theme,
            frame_idx: app.frame_idx,
        };
        frame.render_widget(widget, inner);

        render_footer(frame, footer, &self.theme, app, "space skip   Q quit");
    }
}
