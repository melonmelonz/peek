//! Exponential decay across the three stats with per-stat half-lives.

use crate::Stats;

#[derive(Debug, Clone, Copy)]
pub struct DecayRates {
    pub nourishment_h: f32,
    pub tether_h: f32,
    pub lucidity_h: f32,
}

impl DecayRates {
    pub const DEFAULT: Self = Self {
        nourishment_h: 36.0,
        tether_h: 72.0,
        lucidity_h: 60.0,
    };
}

impl Default for DecayRates {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Apply exponential decay to `stats` over `elapsed_seconds`.
///
/// Each stat halves over its configured half-life. Negative or zero elapsed
/// time is a no-op.
pub fn apply_decay(stats: &mut Stats, rates: DecayRates, elapsed_seconds: f64) {
    if !elapsed_seconds.is_finite() || elapsed_seconds <= 0.0 {
        return;
    }
    let h = (elapsed_seconds / 3600.0) as f32;
    let mul = |half: f32| (-h * std::f32::consts::LN_2 / half).exp();
    stats.nourishment *= mul(rates.nourishment_h);
    stats.tether *= mul(rates.tether_h);
    stats.lucidity *= mul(rates.lucidity_h);
    stats.clamp();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_half_life_halves_nourishment() {
        let mut s = Stats::new_full();
        let r = DecayRates::DEFAULT;
        apply_decay(&mut s, r, (r.nourishment_h as f64) * 3600.0);
        assert!(
            (s.nourishment - 0.5).abs() < 0.001,
            "nourishment={}",
            s.nourishment
        );
    }

    #[test]
    fn very_long_decay_floors_at_zero() {
        let mut s = Stats {
            nourishment: 0.01,
            tether: 0.01,
            lucidity: 0.01,
        };
        apply_decay(&mut s, DecayRates::DEFAULT, 1_000_000.0);
        assert!(s.nourishment >= 0.0);
        assert!(s.tether >= 0.0);
        assert!(s.lucidity >= 0.0);
    }

    #[test]
    fn zero_elapsed_is_noop() {
        let mut s = Stats::new_full();
        apply_decay(&mut s, DecayRates::DEFAULT, 0.0);
        assert_eq!(s, Stats::new_full());
    }

    #[test]
    fn nan_elapsed_is_noop() {
        let mut s = Stats::new_full();
        apply_decay(&mut s, DecayRates::DEFAULT, f64::NAN);
        assert_eq!(s, Stats::new_full());
    }
}
