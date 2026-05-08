//! Visible mutations layered onto the base sprite as the creature stages up.

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutation {
    ExtraEye,
    ThirdMouth,
    Tendril { count: u8 },
    InvertedSpiral,
    Crown,
    NoneAtAll,
}

impl Mutation {
    pub fn label(&self) -> &'static str {
        match self {
            Mutation::ExtraEye => "extra eye",
            Mutation::ThirdMouth => "third mouth",
            Mutation::Tendril { .. } => "tendril",
            Mutation::InvertedSpiral => "inverted spiral",
            Mutation::Crown => "crown",
            Mutation::NoneAtAll => "none at all",
        }
    }

    /// Roll a single mutation drawn from the given RNG.
    pub fn roll(rng: &mut impl Rng) -> Mutation {
        const POOL: &[Mutation] = &[
            Mutation::ExtraEye,
            Mutation::ThirdMouth,
            Mutation::InvertedSpiral,
            Mutation::Crown,
            Mutation::NoneAtAll,
        ];
        let pick = POOL.choose(rng).copied().unwrap_or(Mutation::NoneAtAll);
        if matches!(pick, Mutation::NoneAtAll) && rng.gen_bool(0.4) {
            Mutation::Tendril { count: rng.gen_range(1..=3) }
        } else {
            pick
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn labels_are_concrete() {
        assert_eq!(Mutation::ExtraEye.label(), "extra eye");
        assert_eq!(Mutation::Tendril { count: 3 }.label(), "tendril");
    }

    #[test]
    fn roll_is_deterministic_per_seed() {
        let mut a = ChaCha20Rng::seed_from_u64(1234);
        let mut b = ChaCha20Rng::seed_from_u64(1234);
        let xs: Vec<_> = (0..10).map(|_| Mutation::roll(&mut a)).collect();
        let ys: Vec<_> = (0..10).map(|_| Mutation::roll(&mut b)).collect();
        assert_eq!(xs, ys);
    }
}
