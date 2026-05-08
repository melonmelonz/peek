//! Concrete scenes.

pub mod death;
pub mod hatch;
pub mod idle;
pub mod quiz;
pub mod read;

pub use death::DeathScene;
pub use hatch::HatchScene;
pub use idle::IdleScene;
pub use quiz::QuizScene;
pub use read::ReadScene;
