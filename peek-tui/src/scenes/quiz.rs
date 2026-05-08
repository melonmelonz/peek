//! Quiz/feed scene: pull a Question, show prompt, accept input, evaluate.
//!
//! Used both for "feed" (banked questions, recall scheduled) and "drill"
//! (procedurally generated, also recall scheduled). The mode determines
//! which source we sample from.

use crate::app::App;
use crate::chrome::{render_footer, render_stats, render_title, split_layout};
use crate::scene::{Scene, SceneAction, SceneId};
use crate::theme::Theme;
use chrono::Utc;
use crossterm::event::{Event, KeyCode};
use peek_core::{
    apply_care,
    question::{AttemptResult, Question, QuestionKind},
    recall::RecallRecord,
    CareAction,
};
use rand::seq::IteratorRandom;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuizMode {
    Feed,
    Drill,
}

#[derive(Debug, Clone, Copy)]
enum Phase {
    Asking,
    Resolved { correct: bool },
}

pub struct QuizScene {
    pub theme: Theme,
    pub mode: QuizMode,
    question: Option<Question>,
    input: String,
    phase: Phase,
    reveal: String,
    was_new: bool,
}

impl QuizScene {
    pub fn new(theme: Theme, mode: QuizMode, app: &mut App) -> Self {
        let (q, was_new) = pick_question(app, mode);
        Self {
            theme,
            mode,
            question: q,
            input: String::new(),
            phase: Phase::Asking,
            reveal: String::new(),
            was_new,
        }
    }

    fn submit(&mut self, app: &mut App) {
        let Some(q) = self.question.clone() else {
            return;
        };
        let result: AttemptResult = q.evaluate(&self.input);
        let correct = result.correct;
        self.reveal = result.reveal.clone();
        self.phase = Phase::Resolved { correct };

        if let Some(c) = app.creature_mut() {
            let care = match self.mode {
                QuizMode::Feed => CareAction::Feed {
                    result: result.clone(),
                    was_new: self.was_new,
                },
                QuizMode::Drill => CareAction::Quiz {
                    result: result.clone(),
                    was_new: self.was_new,
                },
            };
            apply_care(c, care);
        }

        // SM-2 scheduling, but only for banked questions (drill is fresh per-call).
        if self.mode == QuizMode::Feed {
            let now = Utc::now();
            let pos = app.state.recall.iter().position(|r| r.question == q.id);
            match pos {
                Some(i) => app.state.recall[i].update(correct, now),
                None => {
                    let mut rec = RecallRecord::new_for(q.id.clone(), now);
                    rec.update(correct, now);
                    app.state.recall.push(rec);
                }
            }
        }

        let event = if correct {
            "feed_correct"
        } else {
            "feed_wrong"
        };
        app.say(event);
        let _ = app.save();
    }
}

fn pick_question(app: &mut App, mode: QuizMode) -> (Option<Question>, bool) {
    match mode {
        QuizMode::Feed => {
            // Pull due-now records first; otherwise pick any banked question
            // the recall log has not seen.
            let now = Utc::now();
            let due: Vec<&RecallRecord> = peek_core::recall::due_now(&app.state.recall, now);
            if let Some(r) = due.first() {
                let id = r.question.clone();
                let q = app.questions.iter().find(|q| q.id == id).cloned();
                return (q, false);
            }
            let seen: std::collections::HashSet<_> = app
                .state
                .recall
                .iter()
                .map(|r| r.question.clone())
                .collect();
            let candidates: Vec<&Question> = app
                .questions
                .iter()
                .filter(|q| !seen.contains(&q.id))
                .collect();
            let q = candidates.into_iter().choose(&mut app.rng).cloned();
            let was_new = q.is_some();
            if q.is_none() {
                let q = app.questions.iter().choose(&mut app.rng).cloned();
                return (q, false);
            }
            (q, was_new)
        }
        QuizMode::Drill => {
            let stage_idx = app.creature().map(|c| c.stage.index()).unwrap_or(0);
            let target = (stage_idx + 1).min(5);
            let q = app.generators.pick(&mut app.rng, target);
            (q, true)
        }
    }
}

impl Scene for QuizScene {
    fn id(&self) -> SceneId {
        SceneId::Quiz
    }

    fn handle(&mut self, ev: &Event, app: &mut App) -> SceneAction {
        if let Event::Key(k) = ev {
            match (self.phase, k.code) {
                (_, KeyCode::Char('Q')) => return SceneAction::Quit,
                (_, KeyCode::Esc) => return SceneAction::Goto(SceneId::Idle),
                (Phase::Asking, KeyCode::Enter) => {
                    if !self.input.trim().is_empty() {
                        self.submit(app);
                    }
                }
                (Phase::Asking, KeyCode::Backspace) => {
                    self.input.pop();
                }
                (Phase::Asking, KeyCode::Char(c)) => {
                    if self.input.len() < 64 {
                        self.input.push(c);
                    }
                }
                (Phase::Resolved { .. }, KeyCode::Enter)
                | (Phase::Resolved { .. }, KeyCode::Char(' ')) => {
                    return SceneAction::Goto(SceneId::Idle);
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
            .border_style(Style::default().fg(self.theme.dim))
            .title(Span::styled(
                match self.mode {
                    QuizMode::Feed => " feed ",
                    QuizMode::Drill => " drill ",
                },
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::BOLD),
            ));
        let inner = block.inner(body);
        frame.render_widget(block, body);

        let mut lines: Vec<Line> = Vec::new();
        match &self.question {
            None => {
                lines.push(Line::from(Span::styled(
                    "no question available right now. try again later.",
                    Style::default().fg(self.theme.warn),
                )));
            }
            Some(q) => match &q.kind {
                QuestionKind::MultipleChoice {
                    prompt, options, ..
                } => {
                    lines.push(wrap_line(prompt, self.theme.fg));
                    lines.push(Line::from(""));
                    for (i, opt) in options.iter().enumerate() {
                        let letter = ((b'a' + i as u8) as char).to_string();
                        lines.push(Line::from(vec![
                            Span::styled(
                                format!("[{letter}] "),
                                Style::default().fg(self.theme.accent_cyan),
                            ),
                            Span::raw(opt.clone()),
                        ]));
                    }
                }
                QuestionKind::FillBlank { prompt, .. } => {
                    lines.push(wrap_line(prompt, self.theme.fg));
                }
                QuestionKind::ShortNumeric { prompt, .. } => {
                    lines.push(wrap_line(prompt, self.theme.fg));
                }
                QuestionKind::TraceProgram { source, .. } => {
                    for line in source.lines() {
                        lines.push(Line::from(Span::styled(
                            line.to_string(),
                            Style::default().fg(self.theme.accent_mint),
                        )));
                    }
                    lines.push(Line::from(""));
                    lines.push(Line::from(
                        "what does this print? (one line per output line)",
                    ));
                }
            },
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("> ", Style::default().fg(self.theme.accent_violet)),
            Span::styled(
                self.input.clone(),
                Style::default()
                    .fg(self.theme.fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "_",
                Style::default()
                    .fg(self.theme.accent_pink)
                    .add_modifier(Modifier::SLOW_BLINK),
            ),
        ]));

        if let Phase::Resolved { correct } = self.phase {
            lines.push(Line::from(""));
            let label = if correct { "yes." } else { "not quite." };
            let color = if correct {
                self.theme.accent_mint
            } else {
                self.theme.warn
            };
            lines.push(Line::from(Span::styled(
                label.to_string(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(wrap_line(&self.reveal, self.theme.fg));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "press enter to return.",
                Style::default().fg(self.theme.dim),
            )));
        }

        let p = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(p, inner);

        render_footer(
            frame,
            footer,
            &self.theme,
            app,
            "enter submit   esc back   Q quit",
        );
    }
}

fn wrap_line(s: &str, color: ratatui::style::Color) -> Line<'static> {
    Line::from(Span::styled(s.to_string(), Style::default().fg(color)))
}
