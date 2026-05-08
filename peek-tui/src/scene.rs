//! Scene trait and the SceneAction return type.

use crate::app::App;
use crate::input::InputEvent;
use ratatui::layout::Rect;
use ratatui::Frame;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Intro,
    Idle,
    Hatch,
    Quiz,
    Read,
    Death,
    Demo,
    Evolve,
}

pub enum SceneAction {
    Stay,
    Goto(SceneId),
    Quit,
}

pub trait Scene {
    fn id(&self) -> SceneId;
    fn handle(&mut self, ev: &InputEvent, app: &mut App) -> SceneAction;
    /// Animation tick. Called every ~100ms even when no key was pressed.
    fn tick(&mut self, _app: &mut App) -> SceneAction {
        SceneAction::Stay
    }
    fn render(&self, frame: &mut Frame, area: Rect, app: &App);
}
