//! ArcLang in the browser: the SAME compiler core as the CLI, compiled to
//! WebAssembly. The model never leaves the user's machine — compilation,
//! validation, the production gate and the explorer rendering all run
//! client-side. There is no server-side compile endpoint at all.

use arclang::compiler::{Compiler, CompilerConfig};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct Stats {
    requirements: usize,
    components: usize,
    functions: usize,
    traces: usize,
    missions: usize,
    capabilities: usize,
    functional_chains: usize,
}

#[derive(Serialize)]
struct GateFindingOut {
    check: String,
    severity: String,
    message: String,
}

#[derive(Serialize)]
struct GateOut {
    passed: bool,
    requirements_total: usize,
    requirements_verified: usize,
    findings: Vec<GateFindingOut>,
}

#[derive(Serialize)]
struct CompileOutput {
    success: bool,
    errors: Vec<String>,
    warnings: Vec<String>,
    stats: Option<Stats>,
    explorer_html: Option<String>,
    gate: Option<GateOut>,
    /// OMG SysML v2 textual notation (interop subset).
    sysmlv2: Option<String>,
    /// OMG ReqIF 1.0 (requirements exchange).
    reqif: Option<String>,
    /// C99 interface contracts.
    c_headers: Option<String>,
    /// Protobuf (proto3) interface contracts.
    proto: Option<String>,
}

/// Compile ArcLang source; returns a JSON string:
/// `{success, errors, warnings, stats, explorer_html, gate}`.
#[wasm_bindgen]
pub fn compile(source: &str) -> String {
    let output = compile_inner(source);
    serde_json::to_string(&output)
        .unwrap_or_else(|e| format!("{{\"success\":false,\"errors\":[\"serialization: {e}\"]}}"))
}

/// Version of the underlying compiler crate.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn compile_inner(source: &str) -> CompileOutput {
    let mut compiler = Compiler::new(CompilerConfig {
        optimization_level: 0,
        target: "json".to_string(),
    });
    match compiler.compile_string(source) {
        Err(e) => CompileOutput {
            success: false,
            errors: vec![e.to_string()],
            warnings: Vec::new(),
            stats: None,
            explorer_html: None,
            gate: None,
            sysmlv2: None,
            reqif: None,
            c_headers: None,
            proto: None,
        },
        Ok(result) => {
            let model = &result.semantic_model;
            let stats = Stats {
                requirements: model.requirements.len(),
                components: model.components.len(),
                functions: model.functions.len(),
                traces: model.traces.len(),
                missions: model.missions.len(),
                capabilities: model.capabilities.len(),
                functional_chains: model.functional_chains.len(),
            };
            let explorer_html =
                arclang::compiler::arcviz_explorer::generate_explorer_html(model, &result.ast)
                    .ok()
                    .map(|(html, _json)| html);
            let gate_report = arclang::compiler::production_gate::run_gate(
                &result.ast,
                model,
                "ISO26262",
            );
            let gate = GateOut {
                passed: gate_report.passed,
                requirements_total: gate_report.requirements_total,
                requirements_verified: gate_report.requirements_verified,
                findings: gate_report
                    .findings
                    .iter()
                    .map(|f| GateFindingOut {
                        check: f.check.clone(),
                        severity: format!("{:?}", f.severity),
                        message: f.message.clone(),
                    })
                    .collect(),
            };
            CompileOutput {
                success: true,
                errors: Vec::new(),
                warnings: result.warnings,
                stats: Some(stats),
                explorer_html,
                gate: Some(gate),
                sysmlv2: Some(arclang::compiler::sysmlv2_generator::generate_sysmlv2(model)),
                reqif: Some(arclang::compiler::reqif::generate_reqif(model, &result.ast)),
                c_headers: Some(arclang::compiler::c_header_generator::generate_c_headers(model, &result.ast)),
                proto: Some(arclang::compiler::proto_generator::generate_proto(model, &result.ast)),
            }
        }
    }
}
