//! Backend-agnostic input event for PEEK scenes.
//!
//! Both the native CLI (crossterm) and the browser build (ratzilla)
//! adapt their native key events into `Key` before handing them to a
//! `Scene::handle`. Scenes never see crossterm or ratzilla types; that
//! lets the same scene code drive both builds.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Backspace,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Key(Key),
}
