//! SM-2 lite recall scheduling.

use crate::question::QuestionId;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecallRecord {
    pub question: QuestionId,
    pub last_seen: DateTime<Utc>,
    pub interval_hours: f32,
    pub ease: f32,
    pub streak: u16,
    pub next_due: DateTime<Utc>,
}

impl RecallRecord {
    pub const EASE_FLOOR: f32 = 1.3;
    pub const FIRST_INTERVAL_H: f32 = 6.0;
    pub const FAIL_INTERVAL_H: f32 = 1.0;

    pub fn new_for(question: QuestionId, now: DateTime<Utc>) -> Self {
        Self {
            question,
            last_seen: now,
            interval_hours: 0.0,
            ease: 2.5,
            streak: 0,
            next_due: now,
        }
    }

    pub fn update(&mut self, correct: bool, now: DateTime<Utc>) {
        if correct {
            self.streak = self.streak.saturating_add(1);
            if self.streak == 1 {
                self.interval_hours = Self::FIRST_INTERVAL_H;
            } else {
                self.interval_hours *= self.ease;
            }
        } else {
            self.streak = 0;
            self.interval_hours = Self::FAIL_INTERVAL_H;
            self.ease = (self.ease - 0.2).max(Self::EASE_FLOOR);
        }
        self.last_seen = now;
        self.next_due = now + Duration::seconds((self.interval_hours * 3600.0) as i64);
    }
}

/// Records due at or before `now`, ordered by `next_due` ascending.
pub fn due_now(records: &[RecallRecord], now: DateTime<Utc>) -> Vec<&RecallRecord> {
    let mut due: Vec<&RecallRecord> = records.iter().filter(|r| r.next_due <= now).collect();
    due.sort_by_key(|r| r.next_due);
    due
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn first_correct_sets_first_interval() {
        let mut r = RecallRecord::new_for(QuestionId::new("q"), now());
        r.update(true, r.last_seen);
        assert!((r.interval_hours - RecallRecord::FIRST_INTERVAL_H).abs() < 0.001);
    }

    #[test]
    fn wrong_answer_floors_ease() {
        let mut r = RecallRecord::new_for(QuestionId::new("q"), now());
        for _ in 0..20 {
            r.update(false, r.last_seen);
        }
        assert!(r.ease >= RecallRecord::EASE_FLOOR);
    }

    #[test]
    fn ease_grows_interval_after_first_correct() {
        let mut r = RecallRecord::new_for(QuestionId::new("q"), now());
        r.update(true, r.last_seen);
        let after_first = r.interval_hours;
        r.update(true, r.last_seen);
        assert!(r.interval_hours > after_first);
    }

    #[test]
    fn due_now_returns_only_records_due() {
        let t = now();
        let mut a = RecallRecord::new_for(QuestionId::new("a"), t);
        let mut b = RecallRecord::new_for(QuestionId::new("b"), t);
        a.update(true, t);
        b.update(true, t);
        let recs = vec![a.clone(), b.clone()];
        let due = due_now(&recs, t);
        assert!(
            due.is_empty(),
            "freshly-correct records should not be due immediately"
        );
        let later = t + Duration::hours(48);
        let recs = vec![a, b];
        let due = due_now(&recs, later);
        assert_eq!(due.len(), 2);
    }
}
