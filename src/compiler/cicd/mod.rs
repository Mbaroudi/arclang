//! CI/CD: what a pipeline needs from the compiler. The command line is in
//! `cli::cicd`; the work is here.
//!
//! - [`diag`]: the compiler's diagnostics and the gate's blockers as one list
//! - [`sarif`]: that list as SARIF 2.1.0 (GitHub code scanning, Azure DevOps)
//! - [`discover`]: which `.arc` files are models (entry points), not fragments
//! - [`pipelines`]: the pipeline files for GitHub, GitLab, Azure DevOps and Jenkins
//! - [`report`]: the pull-request report: semantic diff, impact, gate before and after
//! - [`svg`]: the before and after pictures of a model

pub mod diag;
pub mod discover;
pub mod pipelines;
pub mod report;
pub mod sarif;
pub mod svg;
