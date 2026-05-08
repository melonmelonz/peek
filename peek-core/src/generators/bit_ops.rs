//! Bit-twiddling question generator.
//!
//! Picks a u32 starting value and 1-3 ops from {<<, >>, &, |, ^} with random
//! rhs, asks for the final value as decimal.

use crate::chapter::ChapterId;
use crate::generators::registry::QuestionGenerator;
use crate::question::{Difficulty, Question, QuestionId, QuestionKind};
use rand::Rng;
use rand::RngCore;

#[derive(Default)]
pub struct BitOpsGen;

impl BitOpsGen {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Copy)]
enum Op {
    Shl,
    Shr,
    And,
    Or,
    Xor,
}

impl Op {
    fn pick(rng: &mut dyn RngCore) -> Op {
        match rng.gen_range(0u8..5) {
            0 => Op::Shl,
            1 => Op::Shr,
            2 => Op::And,
            3 => Op::Or,
            _ => Op::Xor,
        }
    }
    fn token(self) -> &'static str {
        match self {
            Op::Shl => "<<",
            Op::Shr => ">>",
            Op::And => "&",
            Op::Or => "|",
            Op::Xor => "^",
        }
    }
    fn apply(self, lhs: u32, rhs: u32) -> u32 {
        match self {
            Op::Shl => lhs.wrapping_shl(rhs & 31),
            Op::Shr => lhs.wrapping_shr(rhs & 31),
            Op::And => lhs & rhs,
            Op::Or => lhs | rhs,
            Op::Xor => lhs ^ rhs,
        }
    }
    fn rhs(self, rng: &mut dyn RngCore) -> u32 {
        match self {
            Op::Shl | Op::Shr => rng.gen_range(0u32..16),
            _ => rng.gen_range(0u32..256),
        }
    }
}

impl QuestionGenerator for BitOpsGen {
    fn id(&self) -> &'static str {
        "bit_ops"
    }

    fn difficulty_range(&self) -> (u8, u8) {
        (1, 5)
    }

    fn generate(&self, rng: &mut dyn RngCore, target_difficulty: u8) -> Question {
        let n_ops = match target_difficulty {
            1 => 1,
            2 | 3 => 2,
            _ => 3,
        };
        let start: u32 = rng.gen_range(0u32..256);
        let mut value = start;
        let mut prompt = format!("starting with {start} (decimal), apply: ");
        let mut steps: Vec<String> = Vec::new();
        for _ in 0..n_ops {
            let op = Op::pick(rng);
            let rhs = op.rhs(rng);
            value = op.apply(value, rhs);
            steps.push(format!("{} {}", op.token(), rhs));
        }
        prompt.push_str(&steps.join(", "));
        prompt.push_str(". what is the final value as a decimal u32?");
        let answer = value as f64;
        Question {
            id: QuestionId::new(format!("gen-bitops-{}", rng.next_u64())),
            chapter: ChapterId::new("ch02-syscalls"),
            difficulty: Difficulty(target_difficulty),
            kind: QuestionKind::ShortNumeric {
                prompt,
                accept_min: answer,
                accept_max: answer,
            },
            explanation: format!(
                "step through each op left to right, treating the value as a u32: starting at {start}, you end at {value}."
            ),
            tags: vec!["generated".into(), "bit-ops".into()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn computed_answer_matches() {
        let g = BitOpsGen::new();
        for seed in 0..100u64 {
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let q = g.generate(&mut rng, 3);
            if let QuestionKind::ShortNumeric { accept_min, .. } = &q.kind {
                let answer_str = format!("{}", *accept_min as u32);
                assert!(q.evaluate(&answer_str).correct, "seed {seed} failed");
            } else {
                panic!("expected ShortNumeric");
            }
        }
    }
}
