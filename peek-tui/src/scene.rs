//! Scene trait and the SceneAction return type.

use crate::app::App;
use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Idle,
    Hatch,
    Quiz,
    Read,
    Death,
}

pub enum SceneAction {
    Stay,
    Goto(SceneId),
    Quit,
}

pub trait Scene {
    fn id(&self) -> SceneId;
    fn handle(&mut self, ev: &Event, app: &mut App) -> SceneAction;
    /// Animation tick. Called every ~100ms even when no key was pressed.
    fn tick(&mut self, _app: &mut App) -> SceneAction {
        SceneAction::Stay
    }
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
}
