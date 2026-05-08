//! Demo scene: a showcase mode that bypasses pacing.
//!
//! Useful for live demos and screenshots — flip through every stage,
//! every mood, roll mutations, force events. Nothing here mutates the
//! `PeekState` that scenes care about (creature, recall, dialogue) in a
//! way that would corrupt a live save: the demo scene takes a private
//! snapshot of the creature on entry and restores it on exit.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::input::{InputEvent, Key};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::sprite::SpriteWidget;
use crate::theme::Theme;
use chrono::Utc;
use peek_content::SpriteSet;
use peek_core::{Creature, Mood, Mutation, Stage};
use rand::Rng;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

const STAGES: [Stage; 6] = Stage::ORDER;
const MOODS: [Mood; 5] = [
    Mood::Anxious,
    Mood::Lucid,
    Mood::Drifting,
    Mood::Ravenous,
    Mood::Reverent,
];

pub struct DemoScene {
    pub theme: Theme,
    snapshot: Option<Creature>,
    stage_idx: usize,
    mood_idx: usize,
    mutations: Vec<Mutation>,
    /// If true, automatically rotates stage every ~2s for hands-off demos.
    autoplay: bool,
    /// Frames since last auto-step.
    auto_counter: u32,
}

impl DemoScene {
    pub fn new(theme: Theme, app: &App) -> Self {
        let snapshot = app.creature().cloned();
        let stage_idx = snapshot
            .as_ref()
            .map(|c| c.stage.index() as usize)
            .unwrap_or(0);
        let mood_idx = snapshot
            .as_ref()
            .map(|c| MOODS.iter().position(|m| *m == c.mood).unwrap_or(0))
            .unwrap_or(0);
        let mutations = snapshot
            .as_ref()
            .map(|c| c.mutations.clone())
            .unwrap_or_default();
        Self {
            theme,
            snapshot,
            stage_idx,
            mood_idx,
            mutations,
            autoplay: false,
            auto_counter: 0,
        }
    }

    fn current_stage(&self) -> Stage {
        STAGES[self.stage_idx % STAGES.len()]
    }

    fn current_mood(&self) -> Mood {
        MOODS[self.mood_idx % MOODS.len()]
    }
}

impl Scene for DemoScene {
    fn id(&self) -> SceneId {
        SceneId::Demo
    }

    fn handle(&mut self, ev: &InputEvent, app: &mut App) -> SceneAction {
        let InputEvent::Key(k) = ev;
        match k {
            Key::Char('Q') => SceneAction::Quit,
            Key::Esc | Key::Char('b') => {
                // Restore the snapshot so demo poking does not affect the
                // real creature.
                app.state.creature = self.snapshot.clone();
                let _ = app.save();
                SceneAction::Goto(SceneId::Idle)
            }
            Key::Right | Key::Char(']') => {
                self.stage_idx = (self.stage_idx + 1) % STAGES.len();
                SceneAction::Stay
            }
            Key::Left | Key::Char('[') => {
                self.stage_idx = (self.stage_idx + STAGES.len() - 1) % STAGES.len();
                SceneAction::Stay
            }
            Key::Up => {
                self.mood_idx = (self.mood_idx + 1) % MOODS.len();
                SceneAction::Stay
            }
            Key::Down => {
                self.mood_idx = (self.mood_idx + MOODS.len() - 1) % MOODS.len();
                SceneAction::Stay
            }
            Key::Char('1') => {
                self.stage_idx = 0;
                SceneAction::Stay
            }
            Key::Char('2') => {
                self.stage_idx = 1;
                SceneAction::Stay
            }
            Key::Char('3') => {
                self.stage_idx = 2;
                SceneAction::Stay
            }
            Key::Char('4') => {
                self.stage_idx = 3;
                SceneAction::Stay
            }
            Key::Char('5') => {
                self.stage_idx = 4;
                SceneAction::Stay
            }
            Key::Char('6') => {
                self.stage_idx = 5;
                SceneAction::Stay
            }
            Key::Char('m') => {
                self.mutations.push(Mutation::roll(&mut app.rng));
                if self.mutations.len() > 5 {
                    self.mutations.remove(0);
                }
                SceneAction::Stay
            }
            Key::Char('c') => {
                self.mutations.clear();
                SceneAction::Stay
            }
            Key::Char('a') => {
                self.autoplay = !self.autoplay;
                self.auto_counter = 0;
                SceneAction::Stay
            }
            Key::Char('n') => {
                // Roll a fresh creature into the demo without persisting it.
                let seed: u64 = app.rng.gen();
                let c = Creature::hatch(Utc::now(), seed);
                self.stage_idx = c.stage.index() as usize;
                self.mood_idx = MOODS.iter().position(|m| *m == c.mood).unwrap_or(0);
                self.mutations.clear();
                SceneAction::Stay
            }
            Key::Char(' ') | Key::Enter => {
                // Apply the current showcase to the live creature so the
                // user can keep playing from this state, then return.
                if let Some(c) = app.creature_mut() {
                    c.stage = self.current_stage();
                    c.mood = self.current_mood();
                    c.mutations = self.mutations.clone();
                    c.stage_advanced_at = Some(Utc::now());
                }
                let _ = app.save();
                self.snapshot = app.creature().cloned();
                SceneAction::Goto(SceneId::Idle)
            }
            _ => SceneAction::Stay,
        }
    }

    fn tick(&mut self, app: &mut App) -> SceneAction {
        app.frame_idx = app.frame_idx.wrapping_add(1);
        if self.autoplay {
            self.auto_counter += 1;
            // tick rate is 120ms; ~16 ticks ≈ 2 seconds per stage.
            if self.auto_counter >= 16 {
                self.auto_counter = 0;
                self.stage_idx = (self.stage_idx + 1) % STAGES.len();
                if self.stage_idx == 0 {
                    self.mood_idx = (self.mood_idx + 1) % MOODS.len();
                }
            }
        }
        SceneAction::Stay
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, stats, body, footer) = split_layout(area);
        render_title(frame, title, &self.theme, app);
        render_stats(frame, stats, &self.theme, app);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(body);

        // Sprite panel.
        let stage = self.current_stage();
        let mood = self.current_mood();
        let frame_bit = (app.frame_idx / 8) % 2;
        let sprite_text = SpriteSet::frame(stage, mood, frame_bit)
            .unwrap_or_else(|| "(missing sprite)".to_string());
        let widget = SpriteWidget {
            stage,
            mood,
            frame: frame_bit,
            mutations: &self.mutations,
            sprite_text,
            theme: self.theme,
            frame_idx: app.frame_idx,
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.accent_violet))
            .title(Span::styled(
                if self.autoplay {
                    " demo · autoplay "
                } else {
                    " demo "
                },
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body_chunks[0]);
        frame.render_widget(block, body_chunks[0]);
        frame.render_widget(widget, inner);

        // Control panel.
        let mut_label: String = if self.mutations.is_empty() {
            "(none)".into()
        } else {
            self.mutations
                .iter()
                .map(|m| m.label())
                .collect::<Vec<_>>()
                .join(", ")
        };
        let lines: Vec<Line> = vec![
            Line::from(Span::styled(
                "showcase mode",
                Style::default()
                    .fg(self.theme.accent_violet)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            stat_line("stage", stage.name(), self.theme.accent_pink, self.theme.fg),
            stat_line("mood", mood.name(), self.theme.accent_pink, self.theme.fg),
            stat_line(
                "mutations",
                &mut_label,
                self.theme.accent_pink,
                self.theme.fg,
            ),
            Line::from(""),
            key_row("[1-6]", "jump to stage", &self.theme),
            key_row("[< >]", "step stage", &self.theme),
            key_row("[^ v]", "step mood", &self.theme),
            key_row("[m]", "roll mutation", &self.theme),
            key_row("[c]", "clear mutations", &self.theme),
            key_row("[n]", "fresh hatch (preview)", &self.theme),
            key_row("[a]", "toggle autoplay", &self.theme),
            key_row("[enter]", "apply to creature, return", &self.theme),
            key_row("[b/esc]", "discard, return to idle", &self.theme),
        ];

        let menu = Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.dim))
                .title(Span::styled(
                    " controls ",
                    Style::default()
                        .fg(self.theme.accent_mint)
                        .add_modifier(Modifier::BOLD),
                )),
        );
        frame.render_widget(menu, body_chunks[1]);

        render_footer(
            frame,
            footer,
            &self.theme,
            app,
            "1-6 jump   ←/→ step   m mutate   a auto   enter apply   b back",
        );
    }
}

fn stat_line(
    label: &str,
    value: &str,
    label_color: ratatui::style::Color,
    value_color: ratatui::style::Color,
) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(label_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(value.to_string(), Style::default().fg(value_color)),
    ])
}

fn key_row(key: &str, label: &str, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(key.to_string(), Style::default().fg(theme.accent_cyan)),
        Span::raw(" "),
        Span::styled(label.to_string(), Style::default().fg(theme.fg)),
    ])
}
