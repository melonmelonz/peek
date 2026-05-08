//! Pointer arithmetic question generator.
//!
//! Picks a struct shape (1-4 fields with sizes from {1,2,4,8}), an offset and
//! a base address. Asks: given a `*const Foo` at base, what address is
//! `&(*p).field`?

use crate::chapter::ChapterId;
use crate::generators::registry::QuestionGenerator;
use crate::question::{Difficulty, Question, QuestionId, QuestionKind};
use rand::Rng;
use rand::RngCore;

#[derive(Default)]
pub struct PointerArithmeticGen;

impl PointerArithmeticGen {
    pub fn new() -> Self {
        Self
    }
}

fn pick_size(rng: &mut dyn RngCore) -> u64 {
    [1u64, 2, 4, 8][rng.gen_range(0..4)]
}

/// Compute field offset assuming each field is naturally aligned to its size.
fn field_offset(sizes: &[u64], target_idx: usize) -> u64 {
    let mut off = 0u64;
    for (i, sz) in sizes.iter().enumerate() {
        let align = *sz;
        if off % align != 0 {
            off += align - (off % align);
        }
        if i == target_idx {
            return off;
        }
        off += sz;
    }
    off
}

impl QuestionGenerator for PointerArithmeticGen {
    fn id(&self) -> &'static str {
        "pointer_arithmetic"
    }

    fn difficulty_range(&self) -> (u8, u8) {
        (2, 5)
    }

    fn generate(&self, rng: &mut dyn RngCore, target_difficulty: u8) -> Question {
        let n_fields: usize = match target_difficulty {
            2 => 2,
            3 => 3,
            _ => 4,
        };
        let sizes: Vec<u64> = (0..n_fields).map(|_| pick_size(rng)).collect();
        let target = rng.gen_range(0..n_fields);
        let base: u64 = 0x1000 * rng.gen_range(1u64..16);
        let off = field_offset(&sizes, target);
        let answer = base + off;

        let mut shape = String::from("struct Foo {\n");
        for (i, sz) in sizes.iter().enumerate() {
            let ty = match sz {
                1 => "u8",
                2 => "u16",
                4 => "u32",
                _ => "u64",
            };
            shape.push_str(&format!("    f{i}: {ty},\n"));
        }
        shape.push('}');

        let prompt = format!(
            "given:\n{shape}\n\na pointer p: *const Foo holds 0x{base:x}. assuming each field is naturally aligned to its size and there is no trailing padding, what address does &(*p).f{target} resolve to? answer in hex (with or without 0x)."
        );

        let accept = vec![
            format!("0x{answer:x}"),
            format!("{answer:x}"),
            format!("0x{answer:X}"),
            format!("{answer:X}"),
        ];

        Question {
            id: QuestionId::new(format!("gen-ptr-{}", rng.next_u64())),
            chapter: ChapterId::new("ch03-memory-and-mmap"),
            difficulty: Difficulty(target_difficulty),
            kind: QuestionKind::FillBlank { prompt, accept },
            explanation: format!(
                "field f{target} sits at offset {off} (0x{off:x}) inside Foo because each prior field is rounded up to its alignment. base 0x{base:x} + 0x{off:x} = 0x{answer:x}."
            ),
            tags: vec!["generated".into(), "pointer-arith".into()],
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
        let g = PointerArithmeticGen::new();
        for seed in 0..100u64 {
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let q = g.generate(&mut rng, 3);
            if let QuestionKind::FillBlank { accept, .. } = &q.kind {
                // First accept is 0x{lower-hex} which is the canonical answer.
                let canonical = accept[0].clone();
                assert!(
                    q.evaluate(&canonical).correct,
                    "seed {seed} canonical {canonical}"
                );
            } else {
                panic!("expected FillBlank");
            }
        }
    }

    #[test]
    fn alignment_padding_respected() {
        // u8, u32 -> u32 is at offset 4 because of alignment.
        let off = field_offset(&[1, 4], 1);
        assert_eq!(off, 4);
    }
}
