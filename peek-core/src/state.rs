//! On-disk state: PeekState as a single RON file with atomic writes.

use crate::creature::Creature;
use crate::recall::RecallRecord;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueEvent {
    pub at: DateTime<Utc>,
    pub event: String,
    pub line: String,
}

const SCHEMA_VERSION: u32 = 1;
const RECENT_DIALOGUE_CAP: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeekState {
    pub schema_version: u32,
    pub creature: Option<Creature>,
    #[serde(default)]
    pub recall: Vec<RecallRecord>,
    #[serde(default)]
    pub recent_dialogue: VecDeque<DialogueEvent>,
    pub install_id: Uuid,
}

impl PeekState {
    pub fn new() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            creature: None,
            recall: Vec::new(),
            recent_dialogue: VecDeque::new(),
            install_id: Uuid::new_v4(),
        }
    }

    pub fn push_dialogue(&mut self, ev: DialogueEvent) {
        if self.recent_dialogue.len() >= RECENT_DIALOGUE_CAP {
            self.recent_dialogue.pop_front();
        }
        self.recent_dialogue.push_back(ev);
    }
}

impl Default for PeekState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ron parse error: {0}")]
    RonParse(String),
    #[error("ron serialize error: {0}")]
    RonSerialize(String),
}

impl PeekState {
    /// Load state from `path`. Missing file yields a fresh `PeekState`.
    pub fn load(path: &Path) -> Result<Self, StateError> {
        match std::fs::read_to_string(path) {
            Ok(raw) => ron::from_str(&raw).map_err(|e| StateError::RonParse(e.to_string())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(PeekState::new()),
            Err(e) => Err(StateError::Io(e)),
        }
    }

    /// Atomically save state to `path` (write-temp-then-rename).
    pub fn save(&self, path: &Path) -> Result<(), StateError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let cfg = ron::ser::PrettyConfig::new().depth_limit(8);
        let body = ron::ser::to_string_pretty(self, cfg)
            .map_err(|e| StateError::RonSerialize(e.to_string()))?;
        let tmp = path.with_extension("ron.tmp");
        std::fs::write(&tmp, body)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }
}

/// Return the path PEEK uses for its state file.
///
/// Order of preference: `$XDG_STATE_HOME/peek/state.ron`, then
/// `$HOME/.local/state/peek/state.ron`, then a temp dir as a last resort.
pub fn default_state_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME") {
        return PathBuf::from(xdg).join("peek").join("state.ron");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("peek")
            .join("state.ron");
    }
    std::env::temp_dir().join("peek").join("state.ron")
}

/// Path for the memorial log alongside state.
pub fn default_memorial_path() -> PathBuf {
    let mut p = default_state_path();
    p.set_file_name("memorials.ron");
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creature::Creature;

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!("peek-state-{}", rand::random::<u64>()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("state.ron");

        let mut s = PeekState::new();
        s.creature = Some(Creature::hatch(Utc::now(), 99));
        s.save(&path).unwrap();
        let loaded = PeekState::load(&path).unwrap();
        assert_eq!(loaded.creature.as_ref().map(|c| c.seed), Some(99));
    }

    #[test]
    fn missing_file_yields_default() {
        let path = std::env::temp_dir().join(format!("peek-missing-{}.ron", rand::random::<u64>()));
        let s = PeekState::load(&path).unwrap();
        assert!(s.creature.is_none());
    }

    #[test]
    fn dialogue_ring_buffer_caps() {
        let mut s = PeekState::new();
        for i in 0..200 {
            s.push_dialogue(DialogueEvent {
                at: Utc::now(),
                event: "test".into(),
                line: format!("line {i}"),
            });
        }
        assert!(s.recent_dialogue.len() <= RECENT_DIALOGUE_CAP);
    }
}
