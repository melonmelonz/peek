//! Concrete scenes.

pub mod death;
pub mod demo;
pub mod hatch;
pub mod idle;
pub mod intro;
pub mod quiz;
pub mod read;

pub use death::DeathScene;
pub use demo::DemoScene;
pub use hatch::HatchScene;
pub use idle::IdleScene;
pub use intro::IntroScene;
pub use quiz::QuizScene;
pub use read::ReadScene;
