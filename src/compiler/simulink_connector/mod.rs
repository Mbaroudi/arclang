//! The Capella-Simulink connector, in ArcLang: what the Eclipse connector
//! (Thales, Obeo; EPL 2.0) does from a Capella model, done from an ArcLang
//! model. Elements chosen by their ids become a pivot (`.mlcore`, the file
//! the connector's MATLAB toolbox imports with `CapSLBridge.import`), and
//! the pivot becomes the MATLAB script that builds the Simulink project.
//!
//! Both halves are ports of the connector's, checked against its own tests
//! (`tests/fixtures/simulink_connector/`): the model-to-model transformation
//! (`m2m`, from the Capella XMI; an ArcLang model is written as Capella
//! first) and the generator (`generator`). `mlcore` writes the pivot,
//! `export` is what `arclang export -f simulink-connector` runs.

pub mod export;
pub mod generator;
pub mod java;
pub mod m2m;
pub mod mlcore;
pub mod pivot;

pub use generator::script;
