//! Model-Based Design toward any hardware, in three layers:
//!
//! 1. `cgen`: the behavior (the state machines) as portable C99 that knows
//!    no board: `<model>.h/.c`, `init()` / `step()` over structures the
//!    caller owns;
//! 2. `binding` and `glue`: one I/O interface, `arclang_io.h`, and the
//!    bindings of machine inputs and outputs to its channels, written once in
//!    the model (`io_binding`), checked by the compiler, generated as glue;
//! 3. target packages implement `arclang_io.h` (`runtime/arclang_io_host.c`
//!    is the PC one, for software in the loop).
//!
//! `sil` proves layers 1 and 2 against the model: the C compiled on the
//! host, run with the vectors of each `execute:` test case, compared with the
//! model-level run (`reference`) step by step. See docs/embedded-c.md.

pub mod binding;
pub mod case;
pub mod cgen;
pub mod glue;
pub mod plan;
pub mod reference;
pub mod sil;

use crate::compiler::ast::Model;

/// The step period the C is generated for: the first binding's, else 10 ms.
pub fn default_period_us(bindings: &[binding::Binding]) -> u64 {
    bindings.first().map(|b| b.period_us).unwrap_or(case::DEFAULT_STEP_US)
}

/// What the compiler checks of the embedded declarations: the bindings, the
/// executed cases (their machine, inputs, binding, the variables their
/// expectations read) and the model-level verdict of each case, as warnings.
pub fn check(ast: &Model) -> Result<Vec<String>, String> {
    let executed = sil::cases(ast);
    if ast.io_bindings.is_empty() && executed.is_empty() {
        return Ok(Vec::new());
    }
    let what = if ast.io_bindings.is_empty() { format!("test_case '{}' executes", executed[0].name) } else { format!("io_binding '{}'", ast.io_bindings[0].name) };
    let plan = plan::plan(ast).map_err(|e| format!("{what}, but {e}"))?;
    let bindings = binding::resolve_all(ast, &plan)?;
    let mut warnings: Vec<String> = bindings.iter().flat_map(|b| b.warnings.iter().map(|w| format!("embedded: {w}"))).collect();
    let periods: Vec<u64> = bindings.iter().map(|b| b.period_us).collect();
    if periods.windows(2).any(|w| w[0] != w[1]) {
        warnings.push("embedded: the io_bindings run at different periods; the C is generated for the first one's (define <MODEL>_PERIOD_US for another)".to_string());
    }
    let aliases = reference::aliases(&plan);
    let columns = reference::columns(&plan);
    for test in executed {
        let case = case::read(test, &plan, &bindings).expect("executes")?;
        let texts = crate::compiler::simulation::expects_of(&test.attributes);
        for text in &texts {
            let parsed = crate::compiler::simulation::expects::parse(text, &aliases).map_err(|e| format!("test_case '{}': {e}", test.name))?;
            for variable in crate::compiler::simulation::expects::variables(&parsed) {
                if !columns.contains(&variable) {
                    return Err(format!(
                        "test_case '{}': '{variable}' is no state or action of the machines (a state reads `\"{}\".{}`, an action `\"{}\".action.<its text, other characters _>`)",
                        test.name,
                        plan.machines[case.machine].name,
                        plan.machines[case.machine].states[0].member,
                        plan.machines[case.machine].name
                    ));
                }
            }
        }
        let table = reference::run(&plan, &case.vectors(&plan), case.step_us);
        let values = sil::judge(&texts, &aliases, &table).map_err(|e| format!("test_case '{}': {e}", test.name))?;
        let failing: Vec<String> = values.iter().filter(|v| !v.holds).map(|v| format!("{} (expected {})", v.value, v.expect)).collect();
        if !failing.is_empty() {
            warnings.push(format!("verification: test_case '{}' fails on the model: {}", test.name, failing.join("; ")));
        }
    }
    Ok(warnings)
}

/// `arclang export -f c`: every file, as (name, text), and notes.
pub fn export(ast: &Model) -> Result<(Vec<(String, String)>, Vec<String>), String> {
    let plan = plan::plan(ast)?;
    let bindings = binding::resolve_all(ast, &plan)?;
    let mut files = sil::files(&plan, &bindings, default_period_us(&bindings));
    files.push(("host/arclang_io_host.h".to_string(), sil::HOST_HEADER.to_string()));
    files.push(("host/arclang_io_host.c".to_string(), sil::HOST_SOURCE.to_string()));
    let mut notes: Vec<String> = plan.machines.iter().flat_map(|m| m.notes.iter().map(move |n| format!("state_machine '{}': {n}", m.name))).collect();
    notes.extend(bindings.iter().flat_map(|b| b.warnings.clone()));
    for m in &plan.machines {
        if !m.conditions.is_empty() {
            notes.push(format!("state_machine '{}': {} guard(s) written in prose are inputs the caller sets (conditions)", m.name, m.conditions.len()));
        }
        if !m.actions.is_empty() {
            notes.push(format!("state_machine '{}': {} action(s) written in prose are reported (counted per step), not executed", m.name, m.actions.len()));
        }
    }
    Ok((files, notes))
}
