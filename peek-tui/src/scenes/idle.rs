//! Idle scene: shows the creature, listens for top-level care keys.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::input::{InputEvent, Key};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::sprite::SpriteWidget;
use crate::theme::Theme;
use peek_content::SpriteSet;
use peek_core::{apply_care, CareAction};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub struct IdleScene {
    pub theme: Theme,
}

impl IdleScene {
    pub fn new(theme: Theme) -> Self {
        Self { theme }
    }
}

impl Scene for IdleScene {
    fn id(&self) -> SceneId {
        SceneId::Idle
    }

    fn handle(&mut self, ev: &InputEvent, app: &mut App) -> SceneAction {
        let InputEvent::Key(k) = ev;
        match k {
            Key::Char('Q') | Key::Char('q') => return SceneAction::Quit,
            Key::Char('f') | Key::Char('z') => return SceneAction::Goto(SceneId::Quiz),
            Key::Char('t') => {
                if let Some(c) = app.creature_mut() {
                    apply_care(c, CareAction::Tend);
                }
                app.say("tend");
                let _ = app.save();
            }
            Key::Char('r') => return SceneAction::Goto(SceneId::Read),
            Key::Char('d') => return SceneAction::Goto(SceneId::Demo),
            Key::Char('i') => return SceneAction::Goto(SceneId::Intro),
            Key::Char('?') => {
                app.current_dialogue =
                    Some("f feed   t tend   r read   z drill   d demo   i intro   q/Q quit".into());
            }
            _ => {}
        }
        SceneAction::Stay
    }

    fn tick(&mut self, app: &mut App) -> SceneAction {
        app.frame_idx = app.frame_idx.wrapping_add(1);
        // Light idle dialogue when the creature drifts.
        if app.current_dialogue.is_none() {
            if let Some(c) = app.creature() {
                let say = if c.stats.tether < 0.25 {
                    Some("idle_low_tether")
                } else if c.stats.nourishment < 0.25 {
                    Some("idle_low_nourishment")
                } else {
                    None
                };
                if let Some(ev) = say {
                    app.say(ev);
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
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(body);

        // Left: sprite.
        if let Some(c) = app.creature() {
            let frame_bit = (app.frame_idx / 8) % 2;
            let sprite_text = SpriteSet::frame(c.stage, c.mood, frame_bit)
                .unwrap_or_else(|| "(missing sprite)".to_string());
            let widget = SpriteWidget {
                stage: c.stage,
                mood: c.mood,
                frame: frame_bit,
                mutations: &c.mutations,
                sprite_text,
                theme: self.theme,
                frame_idx: app.frame_idx,
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.dim))
                .title(Span::styled(
                    " creature ",
                    Style::default()
                        .fg(self.theme.accent_pink)
                        .add_modifier(Modifier::BOLD),
                ));
            let inner = block.inner(body_chunks[0]);
            frame.render_widget(block, body_chunks[0]);
            frame.render_widget(widget, inner);
        }

        // Right: care menu.
        let key_style = Style::default().fg(self.theme.accent_cyan);
        let dim_style = Style::default().fg(self.theme.dim);
        let row = |key: &'static str, label: &'static str, style: Style| -> Line<'static> {
            Line::from(vec![
                Span::styled(key, style),
                Span::raw(" "),
                Span::raw(label),
            ])
        };
        let lines = vec![
            Line::from(Span::styled(
                "tend the creature",
                Style::default()
                    .fg(self.theme.accent_violet)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            row("[f]", "feed it a question", key_style),
            row("[t]", "tend (steady the tether)", key_style),
            row("[r]", "read a chapter together", key_style),
            row("[z]", "drill (procedurally generated)", key_style),
            Line::from(""),
            row("[d]", "demo (cycle stages, no save)", key_style),
            row("[i]", "replay intro", key_style),
            Line::from(""),
            row("[?]", "help", dim_style),
            row("[Q]", "quit", dim_style),
        ];
        let menu = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.theme.dim))
                .title(Span::styled(
                    " care ",
                    Style::default()
                        .fg(self.theme.accent_mint)
                        .add_modifier(Modifier::BOLD),
                )),
        );
        frame.render_widget(menu, body_chunks[1]);

        render_footer(frame, footer, &self.theme, app, "f t r z d i ? Q");
    }
}
