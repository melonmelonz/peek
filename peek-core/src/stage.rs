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
