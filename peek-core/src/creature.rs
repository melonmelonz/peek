//! The creature itself: identity, stats, and lifecycle.

use crate::chapter::ChapterId;
use crate::decay::{apply_decay, DecayRates};
use crate::mood::Mood;
use crate::mutation::Mutation;
use crate::stage::Stage;
use crate::stats::Stats;
use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// Window after a stage transition during which mood reads as Reverent.
pub const REVERENT_WINDOW: Duration = Duration::minutes(10);

/// How long all stats must sit at zero before the creature dies.
pub const ZERO_FLOOR_TO_DEATH: Duration = Duration::hours(24);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: Uuid,
    pub seed: u64,
    pub true_name: String,
    pub stage: Stage,
    pub mood: Mood,
    pub mutations: Vec<Mutation>,
    pub stats: Stats,
    pub born_at: DateTime<Utc>,
    pub last_tick: DateTime<Utc>,
    pub correct_recalls: u32,
    pub chapters_read: BTreeSet<ChapterId>,
    pub stage_advanced_at: Option<DateTime<Utc>>,
    pub all_zero_since: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickOutcome {
    pub died: bool,
    pub advanced: bool,
}

impl Creature {
    pub fn hatch(now: DateTime<Utc>, seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let true_name = generate_true_name(&mut rng);
        Self {
            id: Uuid::new_v4(),
            seed,
            true_name,
            stage: Stage::Egg,
            mood: Mood::Anxious,
            mutations: Vec::new(),
            stats: Stats::new_full(),
            born_at: now,
            last_tick: now,
            correct_recalls: 0,
            chapters_read: BTreeSet::new(),
            stage_advanced_at: None,
            all_zero_since: None,
        }
    }

    pub fn refresh_mood(&mut self, now: DateTime<Utc>) {
        let recently_advanced = self
            .stage_advanced_at
            .map(|t| now.signed_duration_since(t) < REVERENT_WINDOW)
            .unwrap_or(false);
        self.mood = Mood::from_stats(&self.stats, recently_advanced);
    }

    pub fn advance_stage(&mut self, now: DateTime<Utc>) -> bool {
        if let Some(next) = self.stage.next() {
            self.stage = next;
            self.stage_advanced_at = Some(now);
            let mut rng = ChaCha20Rng::seed_from_u64(self.seed.wrapping_mul((next.index() as u64) + 1));
            let n = rng.gen_range(0u8..=2);
            for _ in 0..n {
                self.mutations.push(Mutation::roll(&mut rng));
            }
            true
        } else {
            false
        }
    }

    /// Apply wall-clock decay since `last_tick`. Updates `mood` and detects
    /// the death floor.
    pub fn tick(&mut self, now: DateTime<Utc>) -> TickOutcome {
        let elapsed = now
            .signed_duration_since(self.last_tick)
            .num_seconds()
            .max(0) as f64;
        apply_decay(&mut self.stats, DecayRates::DEFAULT, elapsed);
        self.last_tick = now;

        let died = if self.stats.all_zero() {
            let since = self.all_zero_since.get_or_insert(now);
            now.signed_duration_since(*since) >= ZERO_FLOOR_TO_DEATH
        } else {
            self.all_zero_since = None;
            false
        };

        self.refresh_mood(now);
        TickOutcome { died, advanced: false }
    }
}

/// Generate a deterministic eldritch-flavored name from the rng. Two
/// syllables, an apostrophe, then one syllable.
fn generate_true_name(rng: &mut impl Rng) -> String {
    const ONSETS: &[&str] = &["v", "z", "kh", "thr", "n", "y", "q", "x", "sh", "gn"];
    const NUCLEI: &[&str] = &["ae", "y", "u", "o", "i", "a", "e"];
    const CODAS: &[&str] = &["l", "r", "n", "th", "x", "ss", ""];

    let s = |rng: &mut dyn rand::RngCore| {
        let onset = ONSETS[rng.gen_range(0..ONSETS.len())];
        let nucleus = NUCLEI[rng.gen_range(0..NUCLEI.len())];
        let coda = CODAS[rng.gen_range(0..CODAS.len())];
        format!("{onset}{nucleus}{coda}")
    };
    let head = format!("{}{}", s(rng), s(rng));
    let tail = s(rng);
    format!("{head}'{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_yields_same_name() {
        let t = Utc::now();
        let a = Creature::hatch(t, 42);
        let b = Creature::hatch(t, 42);
        assert_eq!(a.true_name, b.true_name);
    }

    #[test]
    fn different_seed_usually_yields_different_name() {
        let t = Utc::now();
        let a = Creature::hatch(t, 1);
        let b = Creature::hatch(t, 2);
        // Not strictly guaranteed, but with the syllable pool the chance is < 1%.
        assert_ne!(a.true_name, b.true_name);
    }

    #[test]
    fn advance_stage_climbs_ladder() {
        let t = Utc::now();
        let mut c = Creature::hatch(t, 1);
        for _ in 0..5 {
            assert!(c.advance_stage(t));
        }
        assert_eq!(c.stage, Stage::Cogent);
        assert!(!c.advance_stage(t));
    }

    #[test]
    fn tick_with_no_elapsed_does_nothing() {
        let t = Utc::now();
        let mut c = Creature::hatch(t, 1);
        let before = c.stats;
        let outcome = c.tick(t);
        assert!(!outcome.died);
        assert_eq!(c.stats, before);
    }

    #[test]
    fn death_requires_zero_floor_persisted() {
        let t = Utc::now();
        let mut c = Creature::hatch(t, 1);
        c.stats.nourishment = 0.0;
        c.stats.tether = 0.0;
        c.stats.lucidity = 0.0;
        // First tick observes the zero floor but does not yet kill.
        let later = t + Duration::hours(1);
        let outcome = c.tick(later);
        assert!(!outcome.died);
        // After 24h+1m at the zero floor, the creature dies.
        let much_later = later + Duration::hours(25);
        c.stats.nourishment = 0.0;
        c.stats.tether = 0.0;
        c.stats.lucidity = 0.0;
        let outcome = c.tick(much_later);
        assert!(outcome.died);
    }
}
