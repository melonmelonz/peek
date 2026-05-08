//! Mood is soft state derived from current stats and recent events.

use crate::Stats;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mood {
    Anxious,
    Lucid,
    Drifting,
    Ravenous,
    Reverent,
}

impl Mood {
    /// Derive mood from current stats. `recently_advanced` flags the
    /// post-stage-up window where the creature is briefly Reverent regardless
    /// of stat values.
    pub fn from_stats(stats: &Stats, recently_advanced: bool) -> Mood {
        if recently_advanced {
            return Mood::Reverent;
        }
        if stats.nourishment < 0.2 {
            return Mood::Ravenous;
        }
        if stats.tether < 0.2 {
            return Mood::Drifting;
        }
        if stats.lucidity > 0.8 {
            return Mood::Lucid;
        }
        Mood::Anxious
    }

    pub fn name(self) -> &'static str {
        match self {
            Mood::Anxious => "anxious",
            Mood::Lucid => "lucid",
            Mood::Drifting => "drifting",
            Mood::Ravenous => "ravenous",
            Mood::Reverent => "reverent",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ravenous_when_hungry() {
        let s = Stats { nourishment: 0.1, tether: 1.0, lucidity: 0.5 };
        assert_eq!(Mood::from_stats(&s, false), Mood::Ravenous);
    }

    #[test]
    fn drifting_when_low_tether() {
        let s = Stats { nourishment: 0.5, tether: 0.1, lucidity: 0.5 };
        assert_eq!(Mood::from_stats(&s, false), Mood::Drifting);
    }

    #[test]
    fn lucid_when_high_lucidity_and_other_stats_fine() {
        let s = Stats { nourishment: 0.5, tether: 0.5, lucidity: 0.9 };
        assert_eq!(Mood::from_stats(&s, false), Mood::Lucid);
    }

    #[test]
    fn reverent_after_advance_overrides_other_stats() {
        let mut s = Stats::new_full();
        s.nourishment = 0.0;
        assert_eq!(Mood::from_stats(&s, true), Mood::Reverent);
    }
}
