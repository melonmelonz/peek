//! Death scene: shown after `Creature::tick` returns `died: true`.
//!
//! Renders the death dialogue. Press `b` to bury, write a memorial, and
//! return a fresh egg.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::theme::Theme;
use chrono::Utc;
use crossterm::event::{Event, KeyCode};
use peek_core::{
    memorial::{append, Memorial},
    Creature,
};
use rand::Rng;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub struct DeathScene {
    pub theme: Theme,
    last_dead: Option<Creature>,
}

impl DeathScene {
    pub fn new(theme: Theme, app: &App) -> Self {
        Self { theme, last_dead: app.creature().cloned() }
    }
}

impl Scene for DeathScene {
    fn id(&self) -> SceneId {
        SceneId::Death
    }

    fn handle(&mut self, ev: &Event, app: &mut App) -> SceneAction {
        if let Event::Key(k) = ev {
            match k.code {
                KeyCode::Char('Q') | KeyCode::Char('q') => return SceneAction::Quit,
                KeyCode::Char('b') => {
                    if let Some(c) = self.last_dead.clone() {
                        let m = Memorial {
                            creature_id: c.id,
                            true_name: c.true_name.clone(),
                            born_at: c.born_at,
                            died_at: Utc::now(),
                            final_stage: c.stage,
                            chapters_read: c.chapters_read.len() as u32,
                        };
                        let _ = append(&app.memorial_path, m);
                    }
                    let seed: u64 = app.rng.gen();
                    app.state.creature = Some(Creature::hatch(Utc::now(), seed));
                    app.current_dialogue = None;
                    let _ = app.save();
                    return SceneAction::Goto(SceneId::Hatch);
                }
                _ => {}
            }
        }
        SceneAction::Stay
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, stats, body, footer) = split_layout(area);
        render_title(frame, title, &self.theme, app);
        render_stats(frame, stats, &self.theme, app);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.warn))
            .title(Span::styled(
                " return to the void ",
                Style::default().fg(self.theme.warn).add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);

        let mut lines: Vec<Line> = Vec::new();
        if let Some(c) = &self.last_dead {
            lines.push(Line::from(Span::styled(
                c.true_name.clone(),
                Style::default().fg(self.theme.accent_violet).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(Span::styled(
                format!("final stage: {}", c.stage.name()),
                Style::default().fg(self.theme.dim),
            )));
            lines.push(Line::from(Span::styled(
                format!("chapters read: {}", c.chapters_read.len()),
                Style::default().fg(self.theme.dim),
            )));
            lines.push(Line::from(""));
        }
        if let Some(line) = &app.current_dialogue {
            lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(self.theme.fg).add_modifier(Modifier::ITALIC),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "the creature has returned to the void.",
                Style::default().fg(self.theme.fg).add_modifier(Modifier::ITALIC),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "press [b] to bury and begin again.   [Q] quit.",
            Style::default().fg(self.theme.accent_pink).add_modifier(Modifier::BOLD),
        )));

        let p = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(p, inner);

        render_footer(frame, footer, &self.theme, app, "b bury   Q quit");
    }
}
