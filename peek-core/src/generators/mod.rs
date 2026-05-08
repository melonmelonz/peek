//! Procedural question generators.

pub mod bit_ops;
pub mod pointer_arithmetic;
pub mod registry;
pub mod syscall_trace;

pub use bit_ops::BitOpsGen;
pub use pointer_arithmetic::PointerArithmeticGen;
pub use registry::{GeneratorRegistry, QuestionGenerator};
pub use syscall_trace::SyscallTraceGen;
