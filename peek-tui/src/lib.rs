//! PEEK TUI scenes and widgets.
//!
//! All TUI logic lives in this crate so that the binary in `peek-cli` is a
//! thin wrapper that wires `peek-core`, `peek-content`, and `peek-tui`
//! together.

pub mod app;
pub mod chrome;
pub mod input;
pub mod scene;
pub mod scene_runner;
pub mod scenes;
pub mod sprite;
pub mod theme;

pub use app::{App, AppEvent};
pub use input::{InputEvent, Key};
pub use scene::{Scene, SceneAction, SceneId};
pub use scene_runner::{RunnerOutcome, SceneRunner};
pub use theme::Theme;
