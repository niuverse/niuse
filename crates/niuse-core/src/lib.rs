//! Domain layer for niuse: the seven durable stores, prompt assembly, the
//! in-band protocol, and write-time policy.
//!
//! No runtime, no provider, no terminal. `niuse-core` is pure enough to test
//! without an API key or a filesystem sandbox.

#![forbid(unsafe_code)]

pub mod brief;
pub mod candidate;
pub mod date;
pub mod model;
pub mod policy;
pub mod prompt;
pub mod store;

pub use brief::Brief;
pub use candidate::{parse, Block, Parsed, StreamFilter};
pub use date::Date;
pub use model::{Goal, JournalDay, LoopItem, MemoryEntry, MemoryKind, PlanStep};
pub use prompt::{
    compose, compose_with_goals, compose_with_plan, estimate_text_tokens, Composition,
};
pub use store::Store;
