//! Care actions and their effects on creature stats.

use crate::creature::Creature;
use crate::question::AttemptResult;

#[derive(Debug, Clone)]
pub enum CareAction {
    Feed { result: AttemptResult, was_new: bool },
    Tend,
    Read { chapter_seen_before: bool },
    Quiz { result: AttemptResult, was_new: bool },
}

/// Apply a care action's stat effects to the creature.
pub fn apply_care(creature: &mut Creature, action: CareAction) {
    match action {
        CareAction::Feed { result, was_new } => {
            if result.correct {
                creature.stats.nourishment += 0.25;
                if was_new {
                    creature.stats.lucidity += 0.05;
                }
                creature.correct_recalls = creature.correct_recalls.saturating_add(1);
            } else {
                creature.stats.tether -= 0.05;
            }
        }
        CareAction::Tend => {
            creature.stats.tether += 0.10;
        }
        CareAction::Read { chapter_seen_before } => {
            if chapter_seen_before {
                creature.stats.tether += 0.05;
            } else {
                creature.stats.lucidity += 0.20;
            }
        }
        CareAction::Quiz { result, was_new } => {
            if result.correct {
                creature.stats.nourishment += 0.20;
                if was_new {
                    creature.stats.lucidity += 0.10;
                }
                creature.correct_recalls = creature.correct_recalls.saturating_add(1);
            } else {
                creature.stats.tether -= 0.05;
            }
        }
    }
    creature.stats.clamp();
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn fresh_creature() -> Creature {
        Creature::hatch(Utc::now(), 1)
    }

    fn correct() -> AttemptResult {
        AttemptResult { correct: true, reveal: "ok".into() }
    }

    fn wrong() -> AttemptResult {
        AttemptResult { correct: false, reveal: "no".into() }
    }

    #[test]
    fn feed_correct_lifts_nourishment() {
        let mut c = fresh_creature();
        c.stats.nourishment = 0.5;
        apply_care(&mut c, CareAction::Feed { result: correct(), was_new: false });
        assert!(c.stats.nourishment > 0.5);
        assert_eq!(c.correct_recalls, 1);
    }

    #[test]
    fn feed_wrong_drops_tether() {
        let mut c = fresh_creature();
        c.stats.tether = 0.5;
        apply_care(&mut c, CareAction::Feed { result: wrong(), was_new: false });
        assert!(c.stats.tether < 0.5);
    }

    #[test]
    fn read_new_chapter_lifts_lucidity() {
        let mut c = fresh_creature();
        c.stats.lucidity = 0.0;
        apply_care(&mut c, CareAction::Read { chapter_seen_before: false });
        assert!(c.stats.lucidity > 0.0);
    }

    #[test]
    fn tend_lifts_tether_only() {
        let mut c = fresh_creature();
        c.stats = crate::Stats { nourishment: 0.5, tether: 0.5, lucidity: 0.5 };
        apply_care(&mut c, CareAction::Tend);
        assert!(c.stats.tether > 0.5);
        assert!((c.stats.nourishment - 0.5).abs() < 0.001);
        assert!((c.stats.lucidity - 0.5).abs() < 0.001);
    }
}
