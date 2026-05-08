//! Application state shared across scenes. Holds the loaded peek-core
//! `PeekState`, the embedded curriculum, and a deterministic rng.

use peek_content::{default_generators, DialogueLine, DialogueLines, QuestionBank};
use peek_core::{
    generators::GeneratorRegistry,
    question::Question,
    state::{default_memorial_path, default_state_path, DialogueEvent, PeekState},
    Creature, Stage, Stats,
};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::path::PathBuf;

pub enum AppEvent {
    Dialogue(String),
    StageAdvanced(Stage),
    Died,
    Hatched,
}

pub struct App {
    pub state: PeekState,
    pub state_path: PathBuf,
    pub memorial_path: PathBuf,
    pub questions: Vec<Question>,
    pub dialogue: Vec<DialogueLine>,
    pub generators: GeneratorRegistry,
    pub rng: ChaCha20Rng,
    pub frame_idx: u8,
    pub current_dialogue: Option<String>,
    pub events: Vec<AppEvent>,
}

impl App {
    pub fn boot() -> anyhow::Result<Self> {
        let state_path = default_state_path();
        let memorial_path = default_memorial_path();
        let state = PeekState::load(&state_path)?;
        let questions = QuestionBank::load()?;
        let dialogue = DialogueLines::load()?;
        let generators = default_generators();
        let rng = ChaCha20Rng::from_entropy();
        Ok(Self {
            state,
            state_path,
            memorial_path,
            questions,
            dialogue,
            generators,
            rng,
            frame_idx: 0,
            current_dialogue: None,
            events: Vec::new(),
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        self.state.save(&self.state_path).map_err(|e| anyhow::anyhow!("save state: {e}"))
    }

    pub fn creature(&self) -> Option<&Creature> {
        self.state.creature.as_ref()
    }

    pub fn creature_mut(&mut self) -> Option<&mut Creature> {
        self.state.creature.as_mut()
    }

    pub fn stats(&self) -> Stats {
        self.creature().map(|c| c.stats).unwrap_or_else(Stats::new_full)
    }

    /// Pick a dialogue line for the given event key. `stage` is matched
    /// when the line declares one; stage=None lines fit any stage.
    pub fn pick_line(&mut self, event: &str, stage: Option<Stage>) -> Option<String> {
        use rand::seq::IteratorRandom;
        let stage_name = stage.map(|s| s.name().to_string());
        let candidates: Vec<&DialogueLine> = self
            .dialogue
            .iter()
            .filter(|l| l.event == event)
            .filter(|l| match (&l.stage, &stage_name) {
                (Some(want), Some(have)) => want == have,
                (Some(_), None) => false,
                (None, _) => true,
            })
            .collect();
        let pick = candidates.into_iter().choose(&mut self.rng)?;
        Some(pick.text.clone())
    }

    pub fn say(&mut self, event: &str) {
        let stage = self.creature().map(|c| c.stage);
        if let Some(text) = self.pick_line(event, stage) {
            self.state.push_dialogue(DialogueEvent {
                at: chrono::Utc::now(),
                event: event.to_string(),
                line: text.clone(),
            });
            self.current_dialogue = Some(text.clone());
            self.events.push(AppEvent::Dialogue(text));
        }
    }
}
