//! Read scene: pick a chapter the creature has not read; render it; press
//! space to mark it read and return.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::input::{InputEvent, Key};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::theme::Theme;
use peek_content::Curriculum;
use peek_core::{apply_care, chapter::ChapterId, CareAction};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub struct ReadScene {
    pub theme: Theme,
    chapter: Option<ChapterId>,
    text: String,
    seen_before: bool,
    scroll: u16,
    finished: bool,
}

impl ReadScene {
    pub fn new(theme: Theme, app: &App) -> Self {
        let (chapter, text, seen_before) = pick_chapter(app);
        Self {
            theme,
            chapter,
            text,
            seen_before,
            scroll: 0,
            finished: false,
        }
    }
}

fn pick_chapter(app: &App) -> (Option<ChapterId>, String, bool) {
    let ids = Curriculum::chapter_ids();
    let creature = match app.creature() {
        Some(c) => c,
        None => {
            return (
                ids.first().cloned(),
                ids.first()
                    .and_then(Curriculum::chapter_text)
                    .unwrap_or_default(),
                false,
            );
        }
    };
    let unread: Vec<&ChapterId> = ids
        .iter()
        .filter(|id| !creature.chapters_read.contains(id))
        .collect();
    if let Some(id) = unread.first() {
        let text = Curriculum::chapter_text(id).unwrap_or_default();
        return ((*id).clone().into(), text, false);
    }
    let id = ids.first().cloned();
    let text = id
        .as_ref()
        .and_then(Curriculum::chapter_text)
        .unwrap_or_default();
    (id, text, true)
}

impl Scene for ReadScene {
    fn id(&self) -> SceneId {
        SceneId::Read
    }

    fn handle(&mut self, ev: &InputEvent, app: &mut App) -> SceneAction {
        let InputEvent::Key(k) = ev;
        match k {
            Key::Char('Q') => SceneAction::Quit,
            Key::Esc | Key::Char('q') => {
                self.finish(app);
                SceneAction::Goto(SceneId::Idle)
            }
            Key::Char(' ') | Key::Down | Key::PageDown => {
                self.scroll = self.scroll.saturating_add(8);
                SceneAction::Stay
            }
            Key::Up | Key::PageUp => {
                self.scroll = self.scroll.saturating_sub(8);
                SceneAction::Stay
            }
            Key::Enter => {
                self.finish(app);
                SceneAction::Goto(SceneId::Idle)
            }
            _ => SceneAction::Stay,
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        let (title, stats, body, footer) = split_layout(area);
        render_title(frame, title, &self.theme, app);
        render_stats(frame, stats, &self.theme, app);

        let title_text = self
            .chapter
            .as_ref()
            .map(|c| format!(" reading: {} ", c.as_str()))
            .unwrap_or_else(|| " reading ".into());
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.dim))
            .title(Span::styled(
                title_text,
                Style::default()
                    .fg(self.theme.accent_violet)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);

        let lines: Vec<Line> = self
            .text
            .lines()
            .map(|l| {
                if let Some(rest) = l.strip_prefix("# ") {
                    Line::from(Span::styled(
                        rest.to_string(),
                        Style::default()
                            .fg(self.theme.accent_pink)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else if let Some(rest) = l.strip_prefix("## ") {
                    Line::from(Span::styled(
                        rest.to_string(),
                        Style::default()
                            .fg(self.theme.accent_mint)
                            .add_modifier(Modifier::BOLD),
                    ))
                } else {
                    Line::from(Span::styled(
                        l.to_string(),
                        Style::default().fg(self.theme.fg),
                    ))
                }
            })
            .collect();
        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));
        frame.render_widget(p, inner);

        render_footer(
            frame,
            footer,
            &self.theme,
            app,
            "space scroll   enter done   esc back",
        );
    }
}

impl ReadScene {
    fn finish(&mut self, app: &mut App) {
        if self.finished {
            return;
        }
        self.finished = true;
        if let (Some(c), Some(id)) = (app.creature_mut(), self.chapter.clone()) {
            c.chapters_read.insert(id.clone());
            apply_care(
                c,
                CareAction::Read {
                    chapter_seen_before: self.seen_before,
                },
            );
        }
        if !self.seen_before {
            app.say("read_new");
        }
        let _ = app.save();
    }
}
