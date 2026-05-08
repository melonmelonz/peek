//! Six-stage growth ladder.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stage {
    Egg,
    Sprout,
    Knot,
    Mawling,
    Conduit,
    Cogent,
}

impl Stage {
    pub const ORDER: [Stage; 6] = [
        Stage::Egg,
        Stage::Sprout,
        Stage::Knot,
        Stage::Mawling,
        Stage::Conduit,
        Stage::Cogent,
    ];

    pub fn index(self) -> u8 {
        Self::ORDER
            .iter()
            .position(|s| *s == self)
            .expect("Stage missing from ORDER") as u8
    }

    pub fn next(self) -> Option<Stage> {
        let i = self.index() as usize + 1;
        Self::ORDER.get(i).copied()
    }

    pub fn name(self) -> &'static str {
        match self {
            Stage::Egg => "egg",
            Stage::Sprout => "sprout",
            Stage::Knot => "knot",
            Stage::Mawling => "mawling",
            Stage::Conduit => "conduit",
            Stage::Cogent => "cogent",
        }
    }

    /// Engagement gates required to advance from this stage to the next.
    /// Egg never advances via this path (hatch is its own ceremony).
    /// Cogent is terminal.
    pub fn advance_gate(self) -> Option<AdvanceGate> {
        match self {
            Stage::Egg => None,
            Stage::Sprout => Some(AdvanceGate {
                recalls: 5,
                chapters: 1,
                min_avg_stat: 0.50,
            }),
            Stage::Knot => Some(AdvanceGate {
                recalls: 12,
                chapters: 3,
                min_avg_stat: 0.55,
            }),
            Stage::Mawling => Some(AdvanceGate {
                recalls: 25,
                chapters: 6,
                min_avg_stat: 0.60,
            }),
            Stage::Conduit => Some(AdvanceGate {
                recalls: 50,
                chapters: 10,
                min_avg_stat: 0.65,
            }),
            Stage::Cogent => None,
        }
    }
}

/// Engagement criteria for crossing a stage boundary. The creature must
/// have answered enough questions correctly, read enough chapters, and
/// be sufficiently cared-for (average stat across the three pillars).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdvanceGate {
    pub recalls: u32,
    pub chapters: usize,
    pub min_avg_stat: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_chain_terminates_at_cogent() {
        let mut s = Stage::Egg;
        let mut hops = 0;
        while let Some(n) = s.next() {
            s = n;
            hops += 1;
        }
        assert_eq!(hops, 5);
        assert_eq!(s, Stage::Cogent);
    }

    #[test]
    fn names_are_lowercase() {
        for s in Stage::ORDER {
            assert_eq!(s.name(), s.name().to_lowercase());
        }
    }
}
