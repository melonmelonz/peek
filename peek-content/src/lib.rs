//! PEEK embedded content: chapters, question bank, sprite art, dialogue.
//!
//! All assets are baked into the binary via `rust-embed`, so the static-musl
//! release artifact runs offline with no companion files.

use peek_core::{
    chapter::ChapterId,
    generators::{BitOpsGen, GeneratorRegistry, PointerArithmeticGen, SyscallTraceGen},
    question::Question,
    Mood, Stage,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "chapters/"]
struct ChapterFiles;

#[derive(RustEmbed)]
#[folder = "questions/"]
struct QuestionFiles;

#[derive(RustEmbed)]
#[folder = "sprites/"]
struct SpriteFiles;

#[derive(RustEmbed)]
#[folder = "dialogue/"]
struct DialogueFiles;

pub struct Curriculum;

impl Curriculum {
    pub fn chapter_ids() -> Vec<ChapterId> {
        let mut ids: Vec<ChapterId> = ChapterFiles::iter()
            .filter(|p| p.ends_with(".md"))
            .map(|p| ChapterId::new(p.trim_end_matches(".md").to_string()))
            .collect();
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        ids
    }

    pub fn chapter_text(id: &ChapterId) -> Option<String> {
        let path = format!("{}.md", id.as_str());
        ChapterFiles::get(&path).map(|f| String::from_utf8_lossy(&f.data).to_string())
    }
}

pub struct QuestionBank;

impl QuestionBank {
    pub fn load() -> anyhow::Result<Vec<Question>> {
        let raw = QuestionFiles::get("bank.ron")
            .ok_or_else(|| anyhow::anyhow!("questions/bank.ron is missing from the binary"))?;
        let s = std::str::from_utf8(&raw.data)?;
        let qs: Vec<Question> = ron::from_str(s)?;
        Ok(qs)
    }
}

pub struct SpriteSet;

impl SpriteSet {
    /// Look up a sprite frame, falling back to `<stage>_anxious_<frame>` when
    /// the (stage, mood) combination is not authored.
    pub fn frame(stage: Stage, mood: Mood, frame_idx: u8) -> Option<String> {
        let primary = format!("{}_{}_{frame_idx}.txt", stage.name(), mood_label(mood));
        if let Some(f) = SpriteFiles::get(&primary) {
            return Some(String::from_utf8_lossy(&f.data).to_string());
        }
        let fallback = format!("{}_anxious_{frame_idx}.txt", stage.name());
        SpriteFiles::get(&fallback).map(|f| String::from_utf8_lossy(&f.data).to_string())
    }
}

fn mood_label(m: Mood) -> &'static str {
    match m {
        Mood::Anxious => "anxious",
        Mood::Lucid => "lucid",
        Mood::Drifting => "drifting",
        Mood::Ravenous => "ravenous",
        Mood::Reverent => "reverent",
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DialogueLine {
    pub event: String,
    #[serde(default)]
    pub stage: Option<String>,
    pub text: String,
    #[serde(default)]
    pub memorial_aware: bool,
}

pub struct DialogueLines;

impl DialogueLines {
    pub fn load() -> anyhow::Result<Vec<DialogueLine>> {
        let raw = DialogueFiles::get("lines.ron")
            .ok_or_else(|| anyhow::anyhow!("dialogue/lines.ron is missing from the binary"))?;
        let s = std::str::from_utf8(&raw.data)?;
        let lines: Vec<DialogueLine> = ron::from_str(s)?;
        Ok(lines)
    }
}

/// Construct the registry of procedural generators that ships with PEEK.
pub fn default_generators() -> GeneratorRegistry {
    let mut r = GeneratorRegistry::new();
    r.register(Box::new(BitOpsGen::new()));
    r.register(Box::new(PointerArithmeticGen::new()));
    r.register(Box::new(SyscallTraceGen::new()));
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapters_present() {
        let ids = Curriculum::chapter_ids();
        assert!(
            ids.len() >= 3,
            "expected at least three chapters, got {}",
            ids.len()
        );
        for id in &ids {
            let body = Curriculum::chapter_text(id).expect("chapter file should exist");
            assert!(!body.trim().is_empty(), "chapter {} is empty", id.as_str());
        }
    }

    #[test]
    fn question_bank_loads_and_self_validates() {
        let qs = QuestionBank::load().expect("bank parses");
        assert!(
            qs.len() >= 18,
            "expected >= 18 seeded questions, got {}",
            qs.len()
        );
        for q in &qs {
            // Every seeded question must validate against its documented-correct answer.
            let canonical = canonical_answer(q);
            let res = q.evaluate(&canonical);
            assert!(
                res.correct,
                "seeded question {} did not accept its canonical answer {:?}",
                q.id.as_str(),
                canonical
            );
        }
    }

    fn canonical_answer(q: &Question) -> String {
        match &q.kind {
            peek_core::QuestionKind::MultipleChoice { correct, .. } => {
                ((b'a' + correct) as char).to_string()
            }
            peek_core::QuestionKind::FillBlank { accept, .. } => {
                accept.first().cloned().unwrap_or_default()
            }
            peek_core::QuestionKind::ShortNumeric { accept_min, .. } => format!("{accept_min}"),
            peek_core::QuestionKind::TraceProgram {
                expected_output, ..
            } => expected_output.clone(),
        }
    }

    #[test]
    fn dialogue_loads_and_covers_required_events() {
        let lines = DialogueLines::load().expect("dialogue parses");
        assert!(
            lines.len() >= 30,
            "expected >= 30 dialogue lines, got {}",
            lines.len()
        );
        let required = [
            "hatch",
            "feed_correct",
            "feed_wrong",
            "tend",
            "idle_low_tether",
            "idle_low_nourishment",
            "stage_up",
            "read_new",
            "death",
        ];
        for ev in required {
            assert!(
                lines.iter().any(|l| l.event == ev),
                "missing dialogue line for event {ev}"
            );
        }
    }

    #[test]
    fn sprite_atlas_covers_required_keys() {
        let required = [
            (Stage::Egg, Mood::Anxious),
            (Stage::Sprout, Mood::Anxious),
            (Stage::Knot, Mood::Anxious),
            (Stage::Mawling, Mood::Anxious),
            (Stage::Conduit, Mood::Anxious),
            (Stage::Cogent, Mood::Anxious),
        ];
        for (stage, mood) in required {
            for frame in 0..2 {
                assert!(
                    SpriteSet::frame(stage, mood, frame).is_some(),
                    "missing sprite for stage={stage:?} mood={mood:?} frame={frame}"
                );
            }
        }
    }

    #[test]
    fn default_generators_produce_questions() {
        use rand::SeedableRng;
        let reg = default_generators();
        for seed in 0..10u64 {
            let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(seed);
            for d in 1u8..=5 {
                let q = reg.pick(&mut rng, d);
                assert!(
                    q.is_some(),
                    "registry returned None for difficulty {d} seed {seed}"
                );
            }
        }
    }
}
