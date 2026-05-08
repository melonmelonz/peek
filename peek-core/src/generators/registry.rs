//! Trait and registry for procedural question generators.

use crate::question::Question;
use rand::RngCore;

/// A procedural question generator.
pub trait QuestionGenerator: Send + Sync {
    fn id(&self) -> &'static str;
    /// Inclusive difficulty range this generator can target (1..=5 typical).
    fn difficulty_range(&self) -> (u8, u8);
    /// Produce a fresh question seeded by the rng at the given target difficulty.
    fn generate(&self, rng: &mut dyn RngCore, target_difficulty: u8) -> Question;
}

#[derive(Default)]
pub struct GeneratorRegistry {
    gens: Vec<Box<dyn QuestionGenerator>>,
    cursor: std::cell::Cell<usize>,
}

impl GeneratorRegistry {
    pub fn new() -> Self {
        Self {
            gens: Vec::new(),
            cursor: std::cell::Cell::new(0),
        }
    }

    pub fn register(&mut self, gen: Box<dyn QuestionGenerator>) {
        self.gens.push(gen);
    }

    pub fn len(&self) -> usize {
        self.gens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gens.is_empty()
    }

    /// Pick a generator that supports `target_difficulty` and produce a question.
    /// Round-robin across eligible generators.
    pub fn pick(&self, rng: &mut dyn RngCore, target_difficulty: u8) -> Option<Question> {
        if self.gens.is_empty() {
            return None;
        }
        let n = self.gens.len();
        for offset in 0..n {
            let idx = (self.cursor.get() + offset) % n;
            let g = &self.gens[idx];
            let (lo, hi) = g.difficulty_range();
            if target_difficulty >= lo && target_difficulty <= hi {
                self.cursor.set((idx + 1) % n);
                return Some(g.generate(rng, target_difficulty));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn empty_registry_returns_none() {
        let r = GeneratorRegistry::new();
        let mut rng = ChaCha20Rng::seed_from_u64(1);
        assert!(r.pick(&mut rng, 1).is_none());
    }
}
