//! localStorage-backed persistence for the web build.
//!
//! Two keys: `peek/state` for the live `PeekState`, `peek/memorials` for
//! the append-only memorial log. Both are RON, matching the on-disk
//! format the native build uses.

use peek_core::{memorial::Memorial, state::PeekState};
use web_sys::Storage;

const STATE_KEY: &str = "peek/state";
const MEMORIAL_KEY: &str = "peek/memorials";

fn storage() -> Option<Storage> {
    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
}

pub fn load_state() -> PeekState {
    let Some(s) = storage() else {
        return PeekState::new();
    };
    match s.get_item(STATE_KEY) {
        Ok(Some(raw)) => ron::from_str::<PeekState>(&raw).unwrap_or_else(|_| PeekState::new()),
        _ => PeekState::new(),
    }
}

pub fn save_state(state: &PeekState) {
    let Some(s) = storage() else { return };
    let cfg = ron::ser::PrettyConfig::new().depth_limit(8);
    if let Ok(body) = ron::ser::to_string_pretty(state, cfg) {
        let _ = s.set_item(STATE_KEY, &body);
    }
}

pub fn load_memorials() -> Vec<Memorial> {
    let Some(s) = storage() else {
        return Vec::new();
    };
    match s.get_item(MEMORIAL_KEY) {
        Ok(Some(raw)) => ron::from_str::<Vec<Memorial>>(&raw).unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub fn save_memorials(all: &[Memorial]) {
    let Some(s) = storage() else { return };
    let cfg = ron::ser::PrettyConfig::new().depth_limit(4);
    if let Ok(body) = ron::ser::to_string_pretty(&all.to_vec(), cfg) {
        let _ = s.set_item(MEMORIAL_KEY, &body);
    }
}

pub fn clear_all() {
    if let Some(s) = storage() {
        let _ = s.remove_item(STATE_KEY);
        let _ = s.remove_item(MEMORIAL_KEY);
    }
}
