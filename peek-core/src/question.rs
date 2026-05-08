//! Question shape and evaluation.

use crate::chapter::ChapterId;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QuestionId(pub String);

impl QuestionId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for QuestionId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Difficulty(pub u8);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: QuestionId,
    pub chapter: ChapterId,
    pub difficulty: Difficulty,
    pub kind: QuestionKind,
    pub explanation: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionKind {
    MultipleChoice {
        prompt: String,
        options: Vec<String>,
        correct: u8,
    },
    FillBlank {
        prompt: String,
        accept: Vec<String>,
    },
    ShortNumeric {
        prompt: String,
        accept_min: f64,
        accept_max: f64,
    },
    TraceProgram {
        source: String,
        expected_output: String,
    },
}

#[derive(Debug, Clone)]
pub struct AttemptResult {
    pub correct: bool,
    pub reveal: String,
}

impl Question {
    pub fn evaluate(&self, answer: &str) -> AttemptResult {
        let trimmed = answer.trim();
        let correct = match &self.kind {
            QuestionKind::MultipleChoice { options, correct, .. } => {
                let lower = trimmed.to_ascii_lowercase();
                let idx = if let Some(c) = lower.chars().next() {
                    if c.is_ascii_alphabetic() {
                        Some((c as u8 - b'a') as usize)
                    } else if c.is_ascii_digit() {
                        Some((c as u8 - b'0') as usize)
                    } else {
                        None
                    }
                } else {
                    None
                };
                idx.map(|i| i < options.len() && i as u8 == *correct).unwrap_or(false)
            }
            QuestionKind::FillBlank { accept, .. } => {
                let lower = trimmed.to_ascii_lowercase();
                accept.iter().any(|a| a.trim().to_ascii_lowercase() == lower)
            }
            QuestionKind::ShortNumeric { accept_min, accept_max, .. } => trimmed
                .parse::<f64>()
                .map(|v| Self::range(*accept_min, *accept_max).contains(&v))
                .unwrap_or(false),
            QuestionKind::TraceProgram { expected_output, .. } => {
                trimmed == expected_output.trim()
            }
        };
        AttemptResult { correct, reveal: self.explanation.clone() }
    }

    fn range(lo: f64, hi: f64) -> RangeInclusive<f64> {
        if lo <= hi {
            lo..=hi
        } else {
            hi..=lo
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mc(prompt: &str, opts: &[&str], correct: u8) -> Question {
        Question {
            id: QuestionId::new("t1"),
            chapter: ChapterId::new("ch01"),
            difficulty: Difficulty(1),
            kind: QuestionKind::MultipleChoice {
                prompt: prompt.into(),
                options: opts.iter().map(|s| s.to_string()).collect(),
                correct,
            },
            explanation: "because".into(),
            tags: vec![],
        }
    }

    #[test]
    fn multiple_choice_letter_match() {
        let q = mc("?", &["a", "b", "c"], 1);
        assert!(q.evaluate("b").correct);
        assert!(q.evaluate("B").correct);
        assert!(!q.evaluate("a").correct);
    }

    #[test]
    fn fill_blank_case_insensitive() {
        let q = Question {
            id: QuestionId::new("t2"),
            chapter: ChapterId::new("ch01"),
            difficulty: Difficulty(2),
            kind: QuestionKind::FillBlank {
                prompt: "the call to read kernel-side memory in BASIC was named ___".into(),
                accept: vec!["peek".into(), "PEEK".into()],
            },
            explanation: "yes.".into(),
            tags: vec![],
        };
        assert!(q.evaluate("peek").correct);
        assert!(q.evaluate("PEEK").correct);
        assert!(!q.evaluate("poke").correct);
    }

    #[test]
    fn short_numeric_range() {
        let q = Question {
            id: QuestionId::new("t3"),
            chapter: ChapterId::new("ch02"),
            difficulty: Difficulty(3),
            kind: QuestionKind::ShortNumeric {
                prompt: "approx page size, in bytes".into(),
                accept_min: 4096.0,
                accept_max: 4096.0,
            },
            explanation: "pages.".into(),
            tags: vec![],
        };
        assert!(q.evaluate("4096").correct);
        assert!(!q.evaluate("4095").correct);
        assert!(!q.evaluate("forty-ninety-six").correct);
    }
}
