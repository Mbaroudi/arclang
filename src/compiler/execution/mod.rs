//! Executing a model, step by step: what the debugger runs.
//!
//! A state machine is executed as written, without inventing anything: its
//! initial states are entered, an event (the trigger of a transition) fires
//! the transitions it enables, and each step reports what was exited, what
//! was done and what was entered. A guard is the author's text: the
//! compiler cannot decide it, so it is a *condition* the reader sets (true
//! or false); a guard written `true` or `false` decides itself.

pub mod machine;

pub use machine::{Machine, Step};
