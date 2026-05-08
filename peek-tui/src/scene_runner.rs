//! Backend-agnostic scene runner.
//!
//! Owns the active scene and the bookkeeping needed to swap scenes when
//! one returns `SceneAction::Goto`. Native and web backends drive the
//! same runner; they only differ in how they obtain input events and
//! drive the render/tick cadence.

use crate::app::App;
use crate::input::InputEvent;
use crate::scene::{Scene, SceneAction, SceneId};
use crate::scenes::{
    quiz::QuizMode, DeathScene, DemoScene, HatchScene, IdleScene, IntroScene, QuizScene, ReadScene,
};
use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::Frame;

pub enum RunnerOutcome {
    Continue,
    Quit,
}

pub struct SceneRunner {
    scene: Box<dyn Scene>,
    theme: Theme,
}

impl SceneRunner {
    pub fn new(scene: Box<dyn Scene>, theme: Theme) -> Self {
        Self { scene, theme }
    }

    pub fn scene_id(&self) -> SceneId {
        self.scene.id()
    }

    pub fn replace_scene(&mut self, scene: Box<dyn Scene>) {
        self.scene = scene;
    }

    pub fn handle(&mut self, ev: &InputEvent, app: &mut App) -> RunnerOutcome {
        let action = self.scene.handle(ev, app);
        self.apply(action, app)
    }

    pub fn tick(&mut self, app: &mut App) -> RunnerOutcome {
        let action = self.scene.tick(app);
        self.apply(action, app)
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, app: &App) {
        self.scene.render(frame, area, app);
    }

    fn apply(&mut self, action: SceneAction, app: &mut App) -> RunnerOutcome {
        match action {
            SceneAction::Stay => RunnerOutcome::Continue,
            SceneAction::Quit => RunnerOutcome::Quit,
            SceneAction::Goto(id) => {
                self.scene = build_scene(id, self.theme, app);
                RunnerOutcome::Continue
            }
        }
    }
}

pub fn build_scene(id: SceneId, theme: Theme, app: &mut App) -> Box<dyn Scene> {
    match id {
        SceneId::Intro => Box::new(IntroScene::new(theme)),
        SceneId::Idle => Box::new(IdleScene::new(theme)),
        SceneId::Hatch => Box::new(HatchScene::new(theme)),
        SceneId::Quiz => Box::new(QuizScene::new(theme, QuizMode::Feed, app)),
        SceneId::Read => Box::new(ReadScene::new(theme, app)),
        SceneId::Death => Box::new(DeathScene::new(theme, app)),
        SceneId::Demo => Box::new(DemoScene::new(theme, app)),
    }
}
