//! The generated C as an FMI 2.0 co-simulation FMU: `modelDescription.xml`,
//! the sources (the behavior, `arclang_fmi2.h`, `fmu.c`, the wrapper of
//! `init()` / `step()` in the `fmi2*` functions) and, when a C compiler is
//! given, the shared library of this computer under `binaries/`.
//!
//! - inputs: the events (Boolean, posted in the first step of a
//!   communication step when true, in the order of the machine), the
//!   conditions (Boolean) and the `in` variables (Real, Integer, Boolean;
//!   a `Float` is a Real);
//! - outputs: the states (Boolean), the actions (Integer: the times run in
//!   the last step), the other variables, the budgets (Boolean);
//! - `fmi2DoStep(t, h)` runs the steps of the behavior whose time falls in
//!   `[t, t + h)` (step k at k * period) with the inputs set at `t`, the
//!   events posted in the first of them; the outputs are those of the last
//!   step run. So at `k * period` the outputs are the model-level row of
//!   the step at `(k - 1) * period`, whatever the communication step.
//!
//! The wrapper allocates its instance through the callbacks of the
//! importer, as FMI requires; the behavior itself still allocates nothing.

use super::cgen::{self, comment};
use super::plan::Plan;
use super::reference::{Table, Vectors};
use crate::compiler::execution::expr::{Direction, Ty};
use std::fmt::Write as _;

/// One variable of the FMU.
#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub reference: u32,
    pub input: bool,
    /// `Real`, `Integer` or `Boolean`.
    pub ty: &'static str,
    pub start: Option<String>,
    pub description: String,
    /// The C expression that reads it from the instance.
    pub read: String,
    /// The C lvalue an input is written to (with its C type).
    pub write: Option<(String, &'static str)>,
}

fn fmi_type(ty: Ty) -> &'static str {
    match ty {
        Ty::Bool => "Boolean",
        Ty::Int => "Integer",
        Ty::Real | Ty::Float => "Real",
    }
}

fn xml(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The variables, inputs first, in the order of the result columns.
pub fn variables(plan: &Plan) -> Vec<Variable> {
    let mut out = Vec::new();
    let mut event_slot = 0usize;
    for m in &plan.machines {
        let i = &m.ident;
        for e in &m.events {
            out.push(Variable {
                name: format!("{i}.{}", e.member),
                reference: 0,
                input: true,
                ty: "Boolean",
                start: Some("false".to_string()),
                description: format!("event \"{}\": occurs in the first step of the communication step while true", e.text),
                read: format!("inst->events[{event_slot}]"),
                write: Some((format!("inst->events[{event_slot}]"), "bool")),
            });
            event_slot += 1;
        }
        for c in &m.conditions {
            out.push(Variable {
                name: format!("{i}.{}", c.member),
                reference: 0,
                input: true,
                ty: "Boolean",
                start: Some("false".to_string()),
                description: format!("condition \"{}\"", c.text),
                read: format!("inst->inputs.{i}.{}", c.ident),
                write: Some((format!("inst->inputs.{i}.{}", c.ident), "bool")),
            });
        }
        for (n, v) in m.program().inputs() {
            let field = format!("inst->inputs.{i}.{}", m.variables[n].ident);
            out.push(Variable {
                name: format!("{i}.{}", m.variables[n].member),
                reference: 0,
                input: true,
                ty: fmi_type(v.ty),
                start: Some(start_of(v.initial)),
                description: format!("in variable {}", v.text),
                read: field.clone(),
                write: Some((field, v.ty.c())),
            });
        }
    }
    for m in &plan.machines {
        let i = &m.ident;
        for s in &m.states {
            out.push(Variable { name: format!("{i}.{}", s.member), reference: 0, input: false, ty: "Boolean", start: None, description: format!("state {}: active", s.path), read: format!("inst->outputs.{i}.{}", s.ident), write: None });
        }
        for a in &m.actions {
            out.push(Variable { name: format!("{i}.{}", a.member), reference: 0, input: false, ty: "Integer", start: None, description: format!("action \"{}\": times run in the last step", a.text), read: format!("inst->outputs.{i}.{}", a.ident), write: None });
        }
        for (n, v) in m.program().variables.iter().enumerate().filter(|(_, v)| v.direction != Direction::In) {
            out.push(Variable {
                name: format!("{i}.{}", m.variables[n].member),
                reference: 0,
                input: false,
                ty: fmi_type(v.ty),
                start: None,
                description: format!("variable {}", v.text),
                read: format!("inst->outputs.{i}.{}", m.variables[n].ident),
                write: None,
            });
        }
        for b in &m.budgets {
            out.push(Variable { name: format!("{i}.{}", b.member), reference: 0, input: false, ty: "Boolean", start: None, description: format!("budget {} of transition {}: overrun", b.written, b.label), read: format!("inst->outputs.{i}.{}", b.ident), write: None });
        }
    }
    for (k, v) in out.iter_mut().enumerate() {
        v.reference = k as u32;
    }
    out
}

fn start_of(value: crate::compiler::execution::expr::Value) -> String {
    use crate::compiler::execution::expr::Value;
    match value {
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Real(r) => format!("{r:?}"),
        Value::Float(f) => format!("{:?}", f as f64),
    }
}

pub fn model_description(plan: &Plan, period_us: u64, guid: &str) -> String {
    let vars = variables(plan);
    let mut out = String::new();
    let _ = writeln!(out, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    let _ = writeln!(
        out,
        "<fmiModelDescription fmiVersion=\"2.0\" modelName=\"{}\" guid=\"{{{guid}}}\" description=\"The state machines of {} as generated C (ArcLang, docs/embedded-c.md)\" generationTool=\"ArcLang\" variableNamingConvention=\"structured\" numberOfEventIndicators=\"0\">",
        xml(&plan.model),
        xml(&plan.model)
    );
    let _ = writeln!(out, "  <CoSimulation modelIdentifier=\"{}\" canHandleVariableCommunicationStepSize=\"true\" canNotUseMemoryManagementFunctions=\"false\">", plan.ident);
    let _ = writeln!(out, "    <SourceFiles>\n      <File name=\"{}.c\"/>\n      <File name=\"fmu.c\"/>\n    </SourceFiles>\n  </CoSimulation>", plan.ident);
    let _ = writeln!(out, "  <DefaultExperiment startTime=\"0\" stepSize=\"{}\"/>", period_us as f64 / 1e6);
    let _ = writeln!(out, "  <ModelVariables>");
    for v in &vars {
        let (causality, variability) = if v.input { ("input", "discrete") } else { ("output", "discrete") };
        let start = v.start.as_ref().map(|s| format!(" start=\"{s}\"")).unwrap_or_default();
        let initial = if v.input { "" } else { " initial=\"calculated\"" };
        let _ = writeln!(
            out,
            "    <ScalarVariable name=\"{}\" valueReference=\"{}\" causality=\"{causality}\" variability=\"{variability}\"{initial} description=\"{}\">\n      <{}{start}/>\n    </ScalarVariable>",
            xml(&v.name),
            v.reference,
            xml(&v.description),
            v.ty
        );
    }
    let _ = writeln!(out, "  </ModelVariables>\n  <ModelStructure>\n    <Outputs>");
    for (k, _) in vars.iter().enumerate().filter(|(_, v)| !v.input) {
        let _ = writeln!(out, "      <Unknown index=\"{}\"/>", k + 1);
    }
    let _ = writeln!(out, "    </Outputs>\n    <InitialUnknowns>");
    for (k, _) in vars.iter().enumerate().filter(|(_, v)| !v.input) {
        let _ = writeln!(out, "      <Unknown index=\"{}\"/>", k + 1);
    }
    let _ = writeln!(out, "    </InitialUnknowns>\n  </ModelStructure>\n</fmiModelDescription>");
    out
}

/// The FMI 2.0 types and calls the wrapper needs, as the standard declares them.
pub const FMI2_HEADER: &str = r#"/* The FMI 2.0 types and the co-simulation functions the wrapper defines,
   with the signatures of the FMI 2.0 standard (fmi-standard.org). Written
   for ArcLang's FMU wrapper: an importer calls them by these names. */
#ifndef ARCLANG_FMI2_H
#define ARCLANG_FMI2_H

#include <stddef.h>

#if defined(_WIN32)
#define FMI2_Export __declspec(dllexport)
#else
#define FMI2_Export __attribute__((visibility("default")))
#endif

typedef void *fmi2Component;
typedef void *fmi2ComponentEnvironment;
typedef void *fmi2FMUstate;
typedef unsigned int fmi2ValueReference;
typedef double fmi2Real;
typedef int fmi2Integer;
typedef int fmi2Boolean;
typedef char fmi2Char;
typedef const fmi2Char *fmi2String;
typedef char fmi2Byte;
#define fmi2True 1
#define fmi2False 0

typedef enum { fmi2OK, fmi2Warning, fmi2Discard, fmi2Error, fmi2Fatal, fmi2Pending } fmi2Status;
typedef enum { fmi2ModelExchange, fmi2CoSimulation } fmi2Type;
typedef enum { fmi2DoStepStatus, fmi2PendingStatus, fmi2LastSuccessfulTime, fmi2Terminated } fmi2StatusKind;

typedef void (*fmi2CallbackLogger)(fmi2ComponentEnvironment, fmi2String, fmi2Status, fmi2String, fmi2String, ...);
typedef void *(*fmi2CallbackAllocateMemory)(size_t, size_t);
typedef void (*fmi2CallbackFreeMemory)(void *);
typedef void (*fmi2StepFinished)(fmi2ComponentEnvironment, fmi2Status);

typedef struct {
    const fmi2CallbackLogger logger;
    const fmi2CallbackAllocateMemory allocateMemory;
    const fmi2CallbackFreeMemory freeMemory;
    const fmi2StepFinished stepFinished;
    const fmi2ComponentEnvironment componentEnvironment;
} fmi2CallbackFunctions;

#endif
"#;

/// `fmu.c`: the `fmi2*` functions over the generated `init()` / `step()`.
pub fn wrapper(plan: &Plan, guid: &str) -> String {
    let p = &plan.ident;
    let vars = variables(plan);
    let events: usize = plan.machines.iter().map(|m| m.events.len()).sum();
    let mut out = format!("/*\n * The FMI 2.0 co-simulation wrapper of model \"{}\".\n * Generated by ArcLang. Do not edit: regenerate. See docs/embedded-c.md.\n */\n#include <stdbool.h>\n#include <stdint.h>\n#include <string.h>\n#include \"arclang_fmi2.h\"\n#include \"{p}.h\"\n\n#define GUID \"{{{guid}}}\"\n#define EVENTS {}U\n\n", comment(&plan.model), events.max(1));
    let _ = writeln!(out, "typedef struct {{\n    {p}_state state;\n    {p}_inputs inputs;\n    {p}_outputs outputs;\n    bool events[EVENTS];\n    fmi2CallbackFunctions callbacks;\n    fmi2Real time;\n    unsigned long next; /* the next step of the behavior, at next * period */\n}} instance;\n");
    // Reset: the behavior, the inputs at their start values.
    let _ = writeln!(out, "static void reset(instance *inst)\n{{\n    uint32_t k;\n    {p}_init(&inst->state);\n    (void)memset(&inst->inputs, 0, sizeof inst->inputs);\n    (void)memset(&inst->outputs, 0, sizeof inst->outputs);\n    for (k = 0U; k < EVENTS; k++) {{\n        inst->events[k] = false;\n    }}");
    for m in &plan.machines {
        for (n, v) in m.program().inputs() {
            let _ = writeln!(out, "    inst->inputs.{}.{} = {};", m.ident, m.variables[n].ident, super::cexpr::constant(v.initial));
        }
    }
    let _ = writeln!(out, "    inst->time = 0.0;\n    inst->next = 0UL;\n}}\n");
    // One step of the behavior, the events posted on the first.
    let _ = writeln!(out, "static void step(instance *inst, bool first)\n{{");
    let mut slot = 0usize;
    for m in &plan.machines {
        if m.events.is_empty() {
            continue;
        }
        let t = format!("{p}_{}", m.ident);
        let _ = writeln!(out, "    inst->inputs.{}.queued = 0U;", m.ident);
        for e in &m.events {
            let _ = writeln!(out, "    if (first && inst->events[{slot}]) {{\n        (void){t}_post(&inst->inputs.{}, {});\n    }}", m.ident, cgen::event_macro(&t, &e.ident));
            slot += 1;
        }
    }
    let _ = writeln!(out, "    {p}_step(&inst->state, &inst->inputs, &inst->outputs);\n}}\n");
    out.push_str(
        r#"FMI2_Export const char *fmi2GetTypesPlatform(void) { return "default"; }
FMI2_Export const char *fmi2GetVersion(void) { return "2.0"; }
FMI2_Export fmi2Status fmi2SetDebugLogging(fmi2Component c, fmi2Boolean on, size_t n, const fmi2String categories[]) { (void)c; (void)on; (void)n; (void)categories; return fmi2OK; }

FMI2_Export fmi2Component fmi2Instantiate(fmi2String name, fmi2Type type, fmi2String guid, fmi2String location, const fmi2CallbackFunctions *callbacks, fmi2Boolean visible, fmi2Boolean logging)
{
    instance *inst;
    (void)name; (void)location; (void)visible; (void)logging;
    if ((type != fmi2CoSimulation) || (callbacks == NULL) || (callbacks->allocateMemory == NULL) || (guid == NULL) || (strcmp(guid, GUID) != 0)) {
        return NULL;
    }
    inst = (instance *)callbacks->allocateMemory(1U, sizeof(instance));
    if (inst == NULL) {
        return NULL;
    }
    (void)memcpy(&inst->callbacks, callbacks, sizeof(fmi2CallbackFunctions));
    reset(inst);
    return inst;
}

FMI2_Export void fmi2FreeInstance(fmi2Component c)
{
    instance *inst = (instance *)c;
    if ((inst != NULL) && (inst->callbacks.freeMemory != NULL)) {
        inst->callbacks.freeMemory(inst);
    }
}

FMI2_Export fmi2Status fmi2SetupExperiment(fmi2Component c, fmi2Boolean tolerance_defined, fmi2Real tolerance, fmi2Real start, fmi2Boolean stop_defined, fmi2Real stop)
{
    (void)tolerance_defined; (void)tolerance; (void)stop_defined; (void)stop;
    ((instance *)c)->time = start;
    return fmi2OK;
}
FMI2_Export fmi2Status fmi2EnterInitializationMode(fmi2Component c) { (void)c; return fmi2OK; }
FMI2_Export fmi2Status fmi2ExitInitializationMode(fmi2Component c) { (void)c; return fmi2OK; }
FMI2_Export fmi2Status fmi2Terminate(fmi2Component c) { (void)c; return fmi2OK; }
FMI2_Export fmi2Status fmi2Reset(fmi2Component c) { reset((instance *)c); return fmi2OK; }

FMI2_Export fmi2Status fmi2DoStep(fmi2Component c, fmi2Real t, fmi2Real h, fmi2Boolean no_set_prior)
{
    instance *inst = (instance *)c;
    double end_us = (t + h) * 1.0e6;
    bool first = true;
    (void)no_set_prior;
    if (h < 0.0) {
        return fmi2Error;
    }
    /* The steps of the behavior whose time is in [t, t + h), with the
       inputs set at t; the events posted in the first of them. */
    while ((((double)inst->next) * (double)PERIOD) < (end_us - 0.5)) {
        step(inst, first);
        first = false;
        inst->next++;
    }
    inst->time = t + h;
    return fmi2OK;
}
FMI2_Export fmi2Status fmi2CancelStep(fmi2Component c) { (void)c; return fmi2Error; }

FMI2_Export fmi2Status fmi2GetStatus(fmi2Component c, const fmi2StatusKind s, fmi2Status *value) { (void)c; (void)s; *value = fmi2OK; return fmi2OK; }
FMI2_Export fmi2Status fmi2GetRealStatus(fmi2Component c, const fmi2StatusKind s, fmi2Real *value) { (void)s; *value = ((instance *)c)->time; return fmi2OK; }
FMI2_Export fmi2Status fmi2GetIntegerStatus(fmi2Component c, const fmi2StatusKind s, fmi2Integer *value) { (void)c; (void)s; (void)value; return fmi2Discard; }
FMI2_Export fmi2Status fmi2GetBooleanStatus(fmi2Component c, const fmi2StatusKind s, fmi2Boolean *value) { (void)c; (void)s; *value = fmi2False; return fmi2OK; }
FMI2_Export fmi2Status fmi2GetStringStatus(fmi2Component c, const fmi2StatusKind s, fmi2String *value) { (void)c; (void)s; (void)value; return fmi2Discard; }

FMI2_Export fmi2Status fmi2GetString(fmi2Component c, const fmi2ValueReference vr[], size_t n, fmi2String value[]) { (void)c; (void)vr; (void)value; return (n == 0U) ? fmi2OK : fmi2Error; }
FMI2_Export fmi2Status fmi2SetString(fmi2Component c, const fmi2ValueReference vr[], size_t n, const fmi2String value[]) { (void)c; (void)vr; (void)value; return (n == 0U) ? fmi2OK : fmi2Error; }
FMI2_Export fmi2Status fmi2GetFMUstate(fmi2Component c, fmi2FMUstate *s) { (void)c; (void)s; return fmi2Error; }
FMI2_Export fmi2Status fmi2SetFMUstate(fmi2Component c, fmi2FMUstate s) { (void)c; (void)s; return fmi2Error; }
FMI2_Export fmi2Status fmi2FreeFMUstate(fmi2Component c, fmi2FMUstate *s) { (void)c; (void)s; return fmi2Error; }
FMI2_Export fmi2Status fmi2SerializedFMUstateSize(fmi2Component c, fmi2FMUstate s, size_t *size) { (void)c; (void)s; (void)size; return fmi2Error; }
FMI2_Export fmi2Status fmi2SerializeFMUstate(fmi2Component c, fmi2FMUstate s, fmi2Byte bytes[], size_t size) { (void)c; (void)s; (void)bytes; (void)size; return fmi2Error; }
FMI2_Export fmi2Status fmi2DeSerializeFMUstate(fmi2Component c, const fmi2Byte bytes[], size_t size, fmi2FMUstate *s) { (void)c; (void)bytes; (void)size; (void)s; return fmi2Error; }
FMI2_Export fmi2Status fmi2GetDirectionalDerivative(fmi2Component c, const fmi2ValueReference u[], size_t nu, const fmi2ValueReference z[], size_t nz, const fmi2Real dv[], fmi2Real dz[]) { (void)c; (void)u; (void)nu; (void)z; (void)nz; (void)dv; (void)dz; return fmi2Error; }
FMI2_Export fmi2Status fmi2SetRealInputDerivatives(fmi2Component c, const fmi2ValueReference vr[], size_t n, const fmi2Integer order[], const fmi2Real value[]) { (void)c; (void)vr; (void)n; (void)order; (void)value; return fmi2Error; }
FMI2_Export fmi2Status fmi2GetRealOutputDerivatives(fmi2Component c, const fmi2ValueReference vr[], size_t n, const fmi2Integer order[], fmi2Real value[]) { (void)c; (void)vr; (void)n; (void)order; (void)value; return fmi2Error; }

"#,
    );
    let out = out.replace("(double)PERIOD", &format!("(double){}", cgen::period_macro(plan)));
    let mut out = out;
    for (fmi, c_types) in [("Real", &["double", "float"][..]), ("Integer", &["int32_t", "uint8_t"][..]), ("Boolean", &["bool"][..])] {
        let ty = format!("fmi2{fmi}");
        let _ = writeln!(out, "FMI2_Export fmi2Status fmi2Get{fmi}(fmi2Component c, const fmi2ValueReference vr[], size_t n, {ty} value[])\n{{\n    const instance *inst = (const instance *)c;\n    size_t k;\n    fmi2Status status = fmi2OK;\n    (void)inst;\n    (void)value;\n    for (k = 0U; k < n; k++) {{\n        switch (vr[k]) {{");
        for v in vars.iter().filter(|v| v.ty == fmi) {
            let read = if fmi == "Boolean" { format!("({} ? fmi2True : fmi2False)", v.read) } else { format!("({ty}){}", v.read) };
            let _ = writeln!(out, "        case {}U: /* {} */\n            value[k] = {read};\n            break;", v.reference, comment(&v.name));
        }
        let _ = writeln!(out, "        default:\n            status = fmi2Error;\n            break;\n        }}\n    }}\n    return status;\n}}\n");
        let _ = writeln!(out, "FMI2_Export fmi2Status fmi2Set{fmi}(fmi2Component c, const fmi2ValueReference vr[], size_t n, const {ty} value[])\n{{\n    instance *inst = (instance *)c;\n    size_t k;\n    fmi2Status status = fmi2OK;\n    (void)value;\n    for (k = 0U; k < n; k++) {{\n        switch (vr[k]) {{");
        for v in vars.iter().filter(|v| v.ty == fmi && v.input) {
            let (field, c_ty) = v.write.as_ref().expect("an input");
            let value = if fmi == "Boolean" { "(value[k] != fmi2False)".to_string() } else { format!("({c_ty})value[k]") };
            let _ = writeln!(out, "        case {}U: /* {} */\n            {field} = {value};\n            break;", v.reference, comment(&v.name));
        }
        let _ = writeln!(out, "        default:\n            status = fmi2Error;\n            break;\n        }}\n    }}\n    (void)inst;\n    return status;\n}}\n");
        let _ = c_types;
    }
    out
}

/// The platform directory of this computer's binary, as importers look for
/// it: FMI 2.0 names `linux64`, `darwin64`, `win64` (and fmpy reads an
/// aarch64 Linux binary from `linux64` too).
pub fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "darwin64"
    } else if cfg!(target_os = "windows") {
        "win64"
    } else {
        "linux64"
    }
}

/// The files of the FMU: (path in the archive, bytes); the sources only
/// (`binary`: the shared library to add under `binaries/<platform>/`).
pub fn files(plan: &Plan, period_us: u64, binary: Option<Vec<u8>>) -> Vec<(String, Vec<u8>)> {
    let generated = cgen::generate(plan, period_us);
    let guid = guid(&generated.source);
    let mut out = vec![
        ("modelDescription.xml".to_string(), model_description(plan, period_us, &guid).into_bytes()),
        (format!("sources/{}", generated.header_name), generated.header.into_bytes()),
        (format!("sources/{}", generated.source_name), generated.source.into_bytes()),
        ("sources/arclang_fmi2.h".to_string(), FMI2_HEADER.as_bytes().to_vec()),
        ("sources/fmu.c".to_string(), wrapper(plan, &guid).into_bytes()),
    ];
    if let Some(binary) = binary {
        let ext = if cfg!(target_os = "macos") { "dylib" } else if cfg!(target_os = "windows") { "dll" } else { "so" };
        out.push((format!("binaries/{}/{}.{ext}", platform(), plan.ident), binary));
    }
    out
}

/// A GUID from the generated behavior: the same C, the same GUID.
pub fn guid(source: &str) -> String {
    use sha2::{Digest, Sha256};
    let h: Vec<u8> = Sha256::digest(source.as_bytes()).to_vec();
    let hex: String = h.iter().take(16).map(|b| format!("{b:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

/// Build the FMU: the sources, and the shared library compiled with
/// `compiler` (`None`: a source FMU only). Returns the archive and a note.
pub fn build(plan: &Plan, period_us: u64, compiler: Option<&str>, work: &std::path::Path) -> Result<(Vec<u8>, Vec<String>), String> {
    let mut notes = Vec::new();
    let sources = files(plan, period_us, None);
    let mut binary = None;
    if let Some(cc) = compiler {
        let dir = work.join("fmu-build");
        std::fs::create_dir_all(&dir).map_err(|e| format!("cannot make {}: {e}", dir.display()))?;
        for (name, bytes) in &sources {
            if let Some(file) = name.strip_prefix("sources/") {
                std::fs::write(dir.join(file), bytes).map_err(|e| e.to_string())?;
            }
        }
        let library = dir.join("model.so");
        let out = std::process::Command::new(cc)
            .args(["-std=c99", "-O2", "-fPIC", "-shared", "-fvisibility=hidden", "-Wall", "-Wextra", "-Werror", "-I"])
            .arg(&dir)
            .arg(dir.join(format!("{}.c", plan.ident)))
            .arg(dir.join("fmu.c"))
            .arg("-o")
            .arg(&library)
            .arg(format!("-D{}={period_us}UL", cgen::period_macro(plan)))
            .output()
            .map_err(|e| format!("cannot run the C compiler '{cc}': {e}"))?;
        if !out.status.success() {
            return Err(format!("{cc} rejected the FMU sources:\n{}", String::from_utf8_lossy(&out.stderr)));
        }
        binary = Some(std::fs::read(&library).map_err(|e| e.to_string())?);
        notes.push(format!("binary for this computer in binaries/{} (compiled with {cc})", platform()));
    } else {
        notes.push("a source FMU: no binary (give a C compiler to add this computer's)".to_string());
    }
    let mut entries = files(plan, period_us, binary);
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok((crate::compiler::tables::xlsx::zip(&entries), notes))
}

/// The inputs of a case as fmpy reads them (`time` then each input, one row
/// per step) and the model-level reference as the FMU gives it (the row of
/// the step at `t - period` at time `t`). `None` when the FMU cannot give
/// the case: two events in one step in another order than the machine's.
pub fn case_csv(plan: &Plan, vectors: &Vectors, table: &Table, machine: usize) -> Option<(String, String)> {
    let vars = variables(plan);
    let m = &plan.machines[machine];
    for k in 0..vectors.steps {
        let queue = &vectors.queue[machine][k];
        let mut sorted = queue.clone();
        sorted.sort();
        sorted.dedup();
        if &sorted != queue {
            return None;
        }
    }
    let period = table.period_us as f64 / 1e6;
    let inputs: Vec<&Variable> = vars.iter().filter(|v| v.input).collect();
    let mut csv = String::from("time");
    for v in &inputs {
        let _ = write!(csv, ",{}", v.name);
    }
    csv.push('\n');
    for k in 0..vectors.steps {
        let _ = write!(csv, "{}", k as f64 * period);
        for (n, mp) in plan.machines.iter().enumerate() {
            for e in 0..mp.events.len() {
                let _ = write!(csv, ",{}", vectors.events[n][e][k] as u8);
            }
            for c in 0..mp.conditions.len() {
                let _ = write!(csv, ",{}", vectors.conditions[n][c][k] as u8);
            }
            for (i, _) in mp.program().inputs() {
                let _ = write!(csv, ",{}", vectors.values[n][i][k].number());
            }
        }
        csv.push('\n');
    }
    let _ = m;
    let outputs: Vec<(usize, &Variable)> = vars.iter().filter(|v| !v.input).filter_map(|v| table.names.iter().position(|n| n == &v.name).map(|c| (c, v))).collect();
    let mut reference = String::from("time");
    for (_, v) in &outputs {
        let _ = write!(reference, ",{}", v.name);
    }
    reference.push('\n');
    for (k, row) in table.rows.iter().enumerate() {
        let _ = write!(reference, "{}", (k + 1) as f64 * period);
        for (c, _) in &outputs {
            let _ = write!(reference, ",{:?}", row[*c]);
        }
        reference.push('\n');
    }
    Some((csv, reference))
}
