//! Append-only memorial log for creatures that have returned to the void.

use crate::stage::Stage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memorial {
    pub creature_id: Uuid,
    pub true_name: String,
    pub born_at: DateTime<Utc>,
    pub died_at: DateTime<Utc>,
    pub final_stage: Stage,
    pub chapters_read: u32,
}

/// Append a memorial to the on-disk RON list at `path`. Creates the file if
/// missing.
pub fn append(path: &Path, m: Memorial) -> std::io::Result<()> {
    let mut all = load_all(path);
    all.push(m);
    let cfg = ron::ser::PrettyConfig::new().depth_limit(4);
    let body = ron::ser::to_string_pretty(&all, cfg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, body)
}

/// Load all memorials. A missing file or unparseable file yields an empty list.
pub fn load_all(path: &Path) -> Vec<Memorial> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    ron::from_str(&raw).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_memorial() {
        let dir = tempdir();
        let path = dir.join("memorials.ron");
        let m = Memorial {
            creature_id: Uuid::new_v4(),
            true_name: "vyl'eth".into(),
            born_at: Utc::now(),
            died_at: Utc::now(),
            final_stage: Stage::Knot,
            chapters_read: 2,
        };
        append(&path, m.clone()).unwrap();
        let all = load_all(&path);
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].true_name, m.true_name);
    }

    fn tempdir() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        let nonce: u64 = rand::random();
        p.push(format!("peek-mem-{nonce:x}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
