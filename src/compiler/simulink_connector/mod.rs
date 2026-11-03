//! The Capella-Simulink connector, in ArcLang: what the Eclipse connector
//! (Thales, Obeo; EPL 2.0) does from a Capella model, done from an ArcLang
//! model. Elements chosen by their ids become a pivot (`.mlcore`, the file
//! the connector's MATLAB toolbox imports with `CapSLBridge.import`), and
//! the pivot becomes the MATLAB script that builds the Simulink project.
//!
//! The generator is a port of the connector's, checked against the
//! connector's own examples and expected scripts
//! (`tests/fixtures/simulink_connector/`).

pub mod generator;
pub mod java;
pub mod pivot;

pub use generator::script;
