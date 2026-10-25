//! Semantic git: a model versioned like code, reviewed and merged like a model.
//!
//! - [`rich`]: the diff of two revisions of a model, by element identity,
//!   with its consequences (gate, coverage, constraints, budgets);
//! - [`render`]: that diff as text, Markdown, HTML or JSON;
//! - [`layout_diff`]: what changed in the layout file, reported apart;
//! - [`history`]: the commits that changed one element;
//! - [`merge`]: a three-way merge by element identity, for `.arc` and
//!   `.layout.json`, usable as a git merge driver;
//! - [`baseline`]: a signed-off state of the model an auditor can verify.

pub mod baseline;
pub mod git;
pub mod history;
pub mod layout_diff;
pub mod merge;
pub mod render;
pub mod rich;
