//! PEEK core domain logic.
//!
//! Pure types and functions describing the creature, its needs, the
//! curriculum question shape, recall scheduling, and persistence.
//!
//! No I/O beyond the small `state` module; no UI; no network.

pub mod care;
pub mod chapter;
pub mod creature;
pub mod decay;
pub mod generators;
pub mod memorial;
pub mod mood;
pub mod mutation;
pub mod question;
pub mod recall;
pub mod stage;
pub mod state;
pub mod stats;

pub use care::{apply_care, CareAction};
pub use chapter::ChapterId;
pub use creature::{Creature, TickOutcome};
pub use decay::{apply_decay, DecayRates};
pub use memorial::Memorial;
pub use mood::Mood;
pub use mutation::Mutation;
pub use question::{AttemptResult, Difficulty, Question, QuestionId, QuestionKind};
pub use recall::{due_now, RecallRecord};
pub use stage::Stage;
pub use state::{PeekState, StateError};
pub use stats::Stats;
