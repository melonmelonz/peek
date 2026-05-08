//! Syscall trace question generator.
//!
//! Picks a small syscall sequence and asks a multiple-choice question about
//! its likely failure mode or return value.

use crate::chapter::ChapterId;
use crate::generators::registry::QuestionGenerator;
use crate::question::{Difficulty, Question, QuestionId, QuestionKind};
use rand::Rng;
use rand::RngCore;

#[derive(Default)]
pub struct SyscallTraceGen;

impl SyscallTraceGen {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Clone, Copy)]
enum Scene {
    OpenReadClose,
    MmapMunmap,
    WriteFsync,
}

impl QuestionGenerator for SyscallTraceGen {
    fn id(&self) -> &'static str {
        "syscall_trace"
    }

    fn difficulty_range(&self) -> (u8, u8) {
        (1, 4)
    }

    fn generate(&self, rng: &mut dyn RngCore, target_difficulty: u8) -> Question {
        let scene = match rng.gen_range(0..3) {
            0 => Scene::OpenReadClose,
            1 => Scene::MmapMunmap,
            _ => Scene::WriteFsync,
        };

        match scene {
            Scene::OpenReadClose => {
                let prompt = "a program calls open(\"/etc/shadow\", O_RDONLY) as an unprivileged user. which call fails first, and how?".to_string();
                let options = vec![
                    "open returns -1 with errno EACCES".into(),
                    "open returns 0 (stdin)".into(),
                    "read returns -1 with errno EBADF".into(),
                    "close returns 0 unconditionally".into(),
                ];
                let correct = 0u8;
                Question {
                    id: QuestionId::new(format!("gen-syscall-{}", rng.next_u64())),
                    chapter: ChapterId::new("ch02-syscalls"),
                    difficulty: Difficulty(target_difficulty),
                    kind: QuestionKind::MultipleChoice { prompt, options, correct },
                    explanation: "the kernel checks file permission bits before returning a file descriptor. an unprivileged read of a root-only file fails at open with EACCES; subsequent read/close never run.".into(),
                    tags: vec!["generated".into(), "syscall".into()],
                }
            }
            Scene::MmapMunmap => {
                let len_kb = rng.gen_range(1u32..16);
                let prompt = format!(
                    "a program calls mmap(NULL, {} * 1024, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0) and writes through the returned pointer, then calls munmap on it. what does the second access through that pointer do?",
                    len_kb
                );
                let options = vec![
                    "returns the previously written value".into(),
                    "raises SIGSEGV (the mapping is gone)".into(),
                    "silently zeros the page".into(),
                    "calls back into the kernel automatically".into(),
                ];
                let correct = 1u8;
                Question {
                    id: QuestionId::new(format!("gen-syscall-{}", rng.next_u64())),
                    chapter: ChapterId::new("ch03-memory-and-mmap"),
                    difficulty: Difficulty(target_difficulty),
                    kind: QuestionKind::MultipleChoice { prompt, options, correct },
                    explanation: "munmap removes the mapping from the process page tables. the next access has no backing VMA and the kernel delivers SIGSEGV.".into(),
                    tags: vec!["generated".into(), "syscall".into()],
                }
            }
            Scene::WriteFsync => {
                let prompt = "a program writes 4096 bytes to a regular file descriptor and then calls fsync on it. what does fsync guarantee that write alone did not?".to_string();
                let options = vec![
                    "the bytes are encrypted".into(),
                    "the write was successful (write didn't already report that)".into(),
                    "the data and file metadata are durable on the underlying storage".into(),
                    "the file is closed".into(),
                ];
                let correct = 2u8;
                Question {
                    id: QuestionId::new(format!("gen-syscall-{}", rng.next_u64())),
                    chapter: ChapterId::new("ch02-syscalls"),
                    difficulty: Difficulty(target_difficulty),
                    kind: QuestionKind::MultipleChoice { prompt, options, correct },
                    explanation: "write only commits to the kernel page cache. fsync forces the cached data and the file's metadata down to durable storage so a power loss cannot lose them.".into(),
                    tags: vec!["generated".into(), "syscall".into()],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn correct_letter_evaluates_correct() {
        let g = SyscallTraceGen::new();
        for seed in 0..50u64 {
            let mut rng = ChaCha20Rng::seed_from_u64(seed);
            let q = g.generate(&mut rng, 2);
            if let QuestionKind::MultipleChoice { correct, .. } = &q.kind {
                let letter = ((b'a' + *correct) as char).to_string();
                assert!(q.evaluate(&letter).correct, "seed {seed} letter {letter}");
            } else {
                panic!("expected MultipleChoice");
            }
        }
    }
}
