//! Three-stat care model: nourishment, tether, lucidity.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    pub nourishment: f32,
    pub tether: f32,
    pub lucidity: f32,
}

impl Stats {
    pub const fn new_full() -> Self {
        Self { nourishment: 1.0, tether: 1.0, lucidity: 1.0 }
    }

    pub fn clamp(&mut self) {
        self.nourishment = self.nourishment.clamp(0.0, 1.0);
        self.tether = self.tether.clamp(0.0, 1.0);
        self.lucidity = self.lucidity.clamp(0.0, 1.0);
    }

    pub fn any_zero(&self) -> bool {
        self.nourishment <= 0.0 || self.tether <= 0.0 || self.lucidity <= 0.0
    }

    pub fn all_zero(&self) -> bool {
        self.nourishment <= 0.0 && self.tether <= 0.0 && self.lucidity <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_to_unit_interval() {
        let mut s = Stats { nourishment: 1.5, tether: -0.2, lucidity: 0.5 };
        s.clamp();
        assert_eq!(s, Stats { nourishment: 1.0, tether: 0.0, lucidity: 0.5 });
    }

    #[test]
    fn any_zero_detects_one_empty_stat() {
        let s = Stats { nourishment: 0.0, tether: 1.0, lucidity: 1.0 };
        assert!(s.any_zero());
        assert!(!Stats::new_full().any_zero());
    }

    #[test]
    fn all_zero_requires_all_three() {
        let mostly_dead = Stats { nourishment: 0.0, tether: 0.0, lucidity: 0.1 };
        assert!(!mostly_dead.all_zero());
        let dead = Stats { nourishment: 0.0, tether: 0.0, lucidity: 0.0 };
        assert!(dead.all_zero());
    }
}
