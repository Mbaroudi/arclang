"""
Core tools: each one is a subcommand of the compiler and returns what it printed.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..compiler.wrapper import (
    BUILD_TARGETS,
    CATALOGUE_FORMATS,
    CONNECTOR_LEAF_FUNCTIONS,
    CONNECTOR_PORT_NAMING,
    CONNECTOR_STRATEGIES,
    DIAGNOSTIC_FORMATS,
    DIFF_FORMATS,
    DIRECTORY_EXPORT_FORMATS,
    EXPORT_FORMATS,
    IMPACT_DIRECTIONS,
    IMPACT_FORMATS,
    IMPACT_LINK_KINDS,
    METHODS,
    REALIZATION_STAGES,
    SAFETY_STANDARDS,
    WORKBENCH_GATE_STANDARDS,
    ArcLangCompiler,
    Run,
    parse_block,
    parse_blockers,
    parse_coverage,
    parse_diff,
    parse_findings,
)
from .common import (
    MODEL_PATH,
    CommandLine,
    InvalidArguments,
    bounded_seconds,
    diff_against,
    listing,
    report,
    resolve_path,
    schema,
)

# Formats whose result is short enough to hand back to the caller in full.
INLINE_EXPORT_LIMIT = 20_000

STANDARD = {
    "type": "string",
    "enum": list(SAFETY_STANDARDS),
    "description": (
        "Safety standard: iso26262 (HARA, ASIL), do178c (DAL, failure conditions), "
        "iec61508 (SIL)"
    ),
    "default": "iso26262",
}

# The time a run of the embedded or simulation tools may take when the call does not
# say, and the most a call may ask (they build, emulate and simulate: minutes).
RUN_SECONDS = 900
RUN_SECONDS_MAX = 7200
RUN_TIMEOUT = {
    "type": "integer",
    "description": f"Seconds the run may take (default {RUN_SECONDS}, at most {RUN_SECONDS_MAX})",
    "default": RUN_SECONDS,
}
GATE_STANDARD = {
    "type": "string",
    "enum": list(WORKBENCH_GATE_STANDARDS),
    "description": "The standard of the production gate in the answer",
    "default": "ISO26262",
}

# Findings of `check` whose rule blocks the production gate (see `arclang rules`).
GATE_BLOCKING_SEVERITY = "error"

TOOLS: List[Tool] = [
    Tool(
        name="arclang_compile",
        description=(
            "Compile an ArcLang model to its JSON form (`arclang build`, same content as "
            "`arclang_export` with format json). WRITES `<model>.json` next to the model, or "
            "`output_path`; to check without writing use arclang_validate. The compiler is strict: any "
            "construct outside the grammar is an error with its line and column. "
            "Returns the counts the compiler prints and, on failure, its error."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "output_path": {
                    "type": "string",
                    "description": "Where to write the compiled output (default: next to the model)",
                },
                "optimize": {
                    "type": "boolean",
                    "description": "Pass --release",
                    "default": False,
                },
                "target": {
                    "type": "string",
                    "enum": list(BUILD_TARGETS),
                    "description": "What is written (`--target`): json (the default), xml or capella",
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_validate",
        description=(
            "Check a model without writing anything (`arclang check --lint`): compile "
            "errors, `metamodel:` and `constraint:` warnings, traceability warnings and "
            "Arcadia methodology advisories. Run it after every edit. With `as_json` the "
            "answer is the compiler's JSON (`--json`): each finding with its `rule` (see "
            "arclang_rules), `severity` (error = gate blocker, warning, info), `element` "
            "and the `fix` it suggests (set an attribute, add a trace). A requirement or "
            "component is listed as having no trace only when no trace reaches it in "
            "either direction; a trace on a function covers its component. `method: cesam` "
            "reports the CESAM rules instead of the Arcadia ones, `safety: true` passes "
            "`--safety`, and `format: sarif` gives the diagnostics as a SARIF 2.1.0 log "
            "(GitHub code scanning, Azure DevOps)."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "as_json": {
                    "type": "boolean",
                    "description": (
                        "Return the findings as JSON (rule, severity, element, message, fix)"
                    ),
                    "default": False,
                },
                "format": {
                    "type": "string",
                    "enum": list(DIAGNOSTIC_FORMATS),
                    "description": "sarif: the diagnostics as a SARIF 2.1.0 log (`--format sarif`)",
                    "default": "text",
                },
                "method": {
                    "type": "string",
                    "enum": list(METHODS),
                    "description": "The method whose rules are reported (default: arcadia, or "
                    "the one the model declares)",
                },
                "safety": {"type": "boolean", "description": "Pass --safety", "default": False},
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_rules",
        description=(
            "The catalog of validation rules (`arclang rules`): what `check` reports, "
            "each rule with its severity, meaning and how to answer it, and the names of "
            "the gate's own checks. Ask for one `rule` (e.g. metamodel.type) to get its "
            "entry alone. This is the reference for the `rule` of a finding."
        ),
        inputSchema=schema(
            {
                "format": {
                    "type": "string",
                    "enum": list(CATALOGUE_FORMATS),
                    "description": "markdown (a table) or json (a list of rules)",
                    "default": "markdown",
                },
                "rule": {
                    "type": "string",
                    "description": (
                        "Only this rule, by id (e.g. trace.requirement); the answer is then JSON"
                    ),
                },
            }
        ),
    ),
    Tool(
        name="arclang_trace_analysis",
        description=(
            "Traceability of a model (`arclang trace --validate`): coverage, issues, "
            "and the trace matrix with each rationale when `matrix` is true. "
            "`suspects: true` lists the suspect traces instead (`--suspects`): those "
            "reviewed whose one end changed since. `review: true` marks traces as reviewed "
            "(`--review`: all, or those `from` / `to` an element), writing the fingerprints "
            "of their two ends as `reviewed: \"...\"`: the answer is the diff, NOT written "
            "unless `write: true`. Review a trace only when the engineer has looked at it."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "matrix": {
                    "type": "boolean",
                    "description": "Also print the traceability matrix",
                    "default": False,
                },
                "suspects": {
                    "type": "boolean",
                    "description": "List the suspect traces instead of validating",
                    "default": False,
                },
                "review": {
                    "type": "boolean",
                    "description": "Mark traces as reviewed (a diff unless `write`)",
                    "default": False,
                },
                "from": {"type": "string", "description": "With review: only the traces from this element"},
                "to": {"type": "string", "description": "With review: only the traces to this element"},
                "write": {
                    "type": "boolean",
                    "description": "With review: write the file. Default: return the diff",
                    "default": False,
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_info",
        description="Counts of a model (`arclang info --metrics`): elements, requirements, "
        "components, functions, traces, traceability coverage.",
        inputSchema=schema({"model_path": MODEL_PATH}, "model_path"),
    ),
    Tool(
        name="arclang_gate",
        description=(
            "Production-readiness gate (`arclang gate --standard <standard>`): PASS or FAIL "
            "against what an industrial design review requires (every requirement satisfied "
            "and verified by a test_case, HARA and ASIL consistency for iso26262, DAL and "
            "failure conditions for do178c, SIL for iec61508, typed attributes, constraints, "
            "timing budgets, ICD completeness). Returns the verdict and each blocker with its "
            "rule. A FAIL is a result, not a tool error. `format: sarif` gives the blockers "
            "as a SARIF 2.1.0 log instead."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "standard": STANDARD,
                "format": {
                    "type": "string",
                    "enum": list(DIAGNOSTIC_FORMATS),
                    "description": "sarif: the blockers as a SARIF 2.1.0 log (`--format sarif`)",
                    "default": "text",
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_realize",
        description=(
            "The realization ladder of a model and the plan to climb it "
            "(`arclang realize --dry-run --json`): for each requirement, what each stage "
            "says (simulation, sil, pil, hdl, soc, board, twin: pass, fail, error, stale, "
            "missing, or not a stage of it), the steps that would produce the missing or "
            "stale stages in order, and the gaps: requirements with no case that simulates "
            "or executes and no twin. By default nothing runs: propose the missing test_case "
            "(simulate: or execute:) or twin for a gap. With `run: true` the climb is REAL "
            "(`arclang realize --json`): each missing or stale stage is run in order (verify, "
            "sil, pil, hdl, soc, twin: OpenModelica, compilers, emulators, minutes each), the "
            "ladder read back after each step, a requirement's climb stopped at its first "
            "failure unless `keep_going`. `board: true` includes the board, which needs the "
            "hardware."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "requirement": {"type": "string", "description": "Only this requirement (id or name)"},
                "up_to": {
                    "type": "string",
                    "enum": list(REALIZATION_STAGES),
                    "description": "Plan (or climb) up to this stage only",
                },
                "run": {
                    "type": "boolean",
                    "description": "Really climb: run the stages. Default: the plan only (--dry-run)",
                    "default": False,
                },
                "keep_going": {
                    "type": "boolean",
                    "description": "With run: go on past a failure or an error",
                    "default": False,
                },
                "board": {
                    "type": "boolean",
                    "description": "Include the board stage (needs the hardware)",
                    "default": False,
                },
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_partition",
        description=(
            "HW/SW partitioning of a model's machines (`arclang partition --dry-run`): the two "
            "runs that measure every machine as C on the processor (`arclang pil`: step time, "
            "footprint) and as VHDL on the logic (`arclang hdl --synth --as-if`: LUT4, Fmax), "
            "and what was measured already (table kind `partition`: what fits its period, what "
            "the model allocates today). By default nothing runs. With `run: true` the two "
            "measurements are REAL (`arclang partition`): the C built and run on the target's "
            "emulator, the VHDL synthesised, placed and routed (the embedded toolchains, "
            "minutes), and the measurements kept beside the model; `as_json` gives them as JSON."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "target": {"type": "string", "description": "Target package of the C, e.g. zephyr/litex_vexriscv"},
                "device": {"type": "string", "description": "ECP5 part of the logic, e.g. LFE5U-25F-6BG381C"},
                "clock_mhz": {"type": "number", "description": "Clock of the logic, MHz"},
                "machine": {"type": "string", "description": "Only this machine (name or id)"},
                "run": {
                    "type": "boolean",
                    "description": "Really measure. Default: the two runs named only (--dry-run)",
                    "default": False,
                },
                "as_json": {
                    "type": "boolean",
                    "description": "With run: the measurements as JSON",
                    "default": False,
                },
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_cesam",
        description=(
            "The model through the CESAM framework (`arclang table --kind cesam_grid` and "
            "`arclang check --json --method cesam`): the operational, functional and "
            "constructional visions x requirements, states, structure, interactions, "
            "scenarios and interfaces, the elements of each view and its maturity (gap: "
            "nothing modelled yet, with the Capella diagram that draws it), and the CESAM "
            "methodology advisories (`cesam.*` rules). Optional `scope`: a level (OA, SA, "
            "LA, PA, EPBS) or an element, the system of interest."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "scope": {"type": "string", "description": "A level (OA, SA, LA, PA, EPBS) or an element: the system of interest"},
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_schedule",
        description=(
            "Tasks and scheduling (`arclang schedule --json`): the io_bindings of a target as "
            "tasks (period, priority), the worst response time of each by fixed-priority "
            "response-time analysis from the step times `arclang partition` measured, or from "
            "`wcet` given here, and whether every task meets its period. A task whose step time "
            "is unknown makes the call FAIL and says how to measure it. By default nothing "
            "runs on a target. With `prove: true` every task is run together on the target's "
            "emulator first (`--prove`: a build and an emulation, minutes), over `cycles` of "
            "the longest period: each period proved kept, the step times under load."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "target": {"type": "string", "description": "Target package (else the bindings')"},
                "wcet": {
                    "type": "object",
                    "description": "Step times given instead of measured: machine name or id -> microseconds",
                    "additionalProperties": {"type": "number"},
                },
                "prove": {
                    "type": "boolean",
                    "description": "Run every task together on the target's emulator first",
                    "default": False,
                },
                "cycles": {
                    "type": "integer",
                    "description": "With prove: cycles of the longest period to run (default 10)",
                },
                "keep_dir": {
                    "type": "string",
                    "description": "With prove: keep the application, the build and the logs here",
                },
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_diff",
        description=(
            "Semantic diff between two versions of a model (`arclang diff --json`). "
            "Elements are compared by stable identity: moving a block is no change, "
            "renaming an element is a modification, not a removal and an addition. "
            "It reports elements, traces and other relationships added, removed or "
            "modified, with each changed attribute (old and new value). A reworded "
            "trace rationale is a change. Element types are metamodel kinds. Two ways to "
            "name the versions: two files (`old_path`, `new_path`), or one file between two "
            "git revisions (`model_path` with `from_rev`, and `to_rev` when it is not the "
            "working tree). `format` (text, json, md, html) asks for the rich report: the "
            "changes by layer and their consequences (the gate and the coverage before and "
            "after, the layout); `output_path` writes it to a file."
        ),
        inputSchema=schema(
            {
                "old_path": {"type": "string", "description": "The old version of the model"},
                "new_path": {"type": "string", "description": "The new version of the model"},
                "model_path": {
                    "type": "string",
                    "description": "With from_rev: the model file compared between revisions",
                },
                "from_rev": {
                    "type": "string",
                    "description": "The file as committed at this git revision (HEAD, main, a tag)...",
                },
                "to_rev": {
                    "type": "string",
                    "description": "...compared with the file at this revision (default: the working tree)",
                },
                "format": {
                    "type": "string",
                    "enum": list(DIFF_FORMATS),
                    "description": "The rich report in this format (default: the JSON of the changes)",
                },
                "output_path": {"type": "string", "description": "Write the report to this file"},
                "gate_standard": GATE_STANDARD,
            }
        ),
    ),
    Tool(
        name="arclang_impact",
        description=(
            "Change-impact analysis (`arclang impact`): everything transitively affected by "
            "a change, by distance, with the relation that carries the impact, its severity "
            "and the gate before and after. The change is one of: one `element` or several "
            "`elements` that change; `change`, a proposed edit that is NEVER written (one "
            "arclang_edit request or a list of them: the what-if); `delete`, an element "
            "removed with the traces naming it; `since`, the changes since a git revision. "
            "`depth`, `kinds` and `direction` bound what is followed; `format` is text, json, "
            "md, html, tsv or xlsx (xlsx needs `output_path`)."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": {
                    "type": "string",
                    "description": 'Element id or unambiguous name, e.g. "REQ-AEB-004"',
                },
                "elements": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Several changed elements: id, unique name or Owner.port",
                },
                "change": {
                    "description": "A proposed change, never written: one arclang_edit request "
                    '({"op": "set", "element": ..., "key": ..., "value": ...}) or a list of them',
                },
                "delete": {
                    "type": "string",
                    "description": "What if this element were deleted, with what names it",
                },
                "since": {
                    "type": "string",
                    "description": "The changes since this git revision (HEAD, main, a commit)",
                },
                "current_path": {
                    "type": "string",
                    "description": "With since: the current text of the model when it is not "
                    "the file on disk",
                },
                "depth": {"type": "integer", "description": "How far the change is followed"},
                "kinds": {
                    "type": "array",
                    "items": {"type": "string", "enum": list(IMPACT_LINK_KINDS)},
                    "description": "The kinds of link followed (default: all)",
                },
                "direction": {
                    "type": "string",
                    "enum": list(IMPACT_DIRECTIONS),
                    "description": "downstream: what depends on the change; upstream: what it "
                    "depends on",
                    "default": "both",
                },
                "gate_standard": GATE_STANDARD,
                "format": {
                    "type": "string",
                    "enum": list(IMPACT_FORMATS),
                    "description": "Format of the answer",
                    "default": "text",
                },
                "output_path": {"type": "string", "description": "Write the answer to this file"},
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_export",
        description=(
            "Export a model (`arclang export -f <format>`). `sys-ml` is the SysML v2 textual "
            "notation, `sysml-json` its abstract syntax as the JSON records of the Systems "
            "Modeling API, `capella` the Capella project (.capella, .aird, .afm), `req-if` "
            "the requirements, `c-headers` and `proto` the interface contracts, `json` the "
            "compiled model, `markdown`, `mermaid`, `plant-uml`, `html` documents, "
            "`geojson` the located elements on a map, `step` the product structure and the "
            "shapes for CAD (STEP AP242), `simulink` and `simulink-connector` (the pivot "
            "`.mlcore` and the MATLAB script of the Capella-Simulink connector; `strategy`, "
            "`elements`, `leaf_functions`, `port_naming`). Formats that fill a DIRECTORY "
            "(`output_path` is then a directory): `c` the portable C99 of the state machines "
            "with the I/O interface, `fmu` that C as an FMI 2.0 co-simulation FMU, `zephyr` "
            "a Zephyr application for a target package (`target`, `binding`), `vhdl` the "
            "VHDL-2008 of the machines allocated to programmable logic, `modelica` a "
            "Modelica package of the physical architecture (`split`, `no_placement`), `fmi` "
            "one FMU interface per component. Short results are returned; longer ones are "
            "only written; a directory is listed. To read a SysML, Capella or ReqIF file "
            "into ArcLang use arclang_import."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "format": {
                    "type": "string",
                    "enum": list(EXPORT_FORMATS),
                    "description": "Export format",
                },
                "output_path": {
                    "type": "string",
                    "description": "Output file, or directory for c, fmu, zephyr, vhdl, modelica "
                    "and fmi (default: nothing is kept, the result is returned or listed)",
                },
                "target": {
                    "type": "string",
                    "description": "zephyr: the target package (default: the io_binding's target)",
                },
                "binding": {
                    "type": "string",
                    "description": "zephyr: the io_binding the controller runs (default: the only one)",
                },
                "split": {
                    "type": "boolean",
                    "description": "modelica: one file per class in a <Package>/ folder",
                    "default": False,
                },
                "no_placement": {
                    "type": "boolean",
                    "description": "modelica: the structure without the drawing annotations",
                    "default": False,
                },
                "strategy": {
                    "type": "string",
                    "enum": list(CONNECTOR_STRATEGIES),
                    "description": "simulink-connector: what to export (default: functions)",
                },
                "elements": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "simulink-connector: the elements to export, by Capella id "
                    "or by name (default: the strategy's roots)",
                },
                "leaf_functions": {
                    "type": "string",
                    "enum": list(CONNECTOR_LEAF_FUNCTIONS),
                    "description": "simulink-connector: leaf functions as a referenced model "
                    "each, or as a subsystem",
                },
                "port_naming": {
                    "type": "string",
                    "enum": list(CONNECTOR_PORT_NAMING),
                    "description": "simulink-connector: ports named after the exchange or "
                    "after the Capella port",
                },
            },
            "model_path",
            "format",
        ),
    ),
    Tool(
        name="arclang_export_diagram",
        description=(
            "Kept for existing clients: same as arclang_export, with `html` as the "
            "default format. For the Arcadia viewpoint diagrams use arclang_viewpoint_diagrams."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "format": {
                    "type": "string",
                    "enum": list(EXPORT_FORMATS),
                    "description": "Export format",
                    "default": "html",
                },
                "output_path": {"type": "string", "description": "Output file path"},
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_metamodel",
        description=(
            "The language metamodel as the compiler holds it (`arclang metamodel`): element "
            "kinds, typed attributes, enumerations, units, trace rules. Ask for one `kind` "
            "(e.g. LogicalComponent) to get only its attributes. This is the reference for "
            "which attribute a kind accepts and with which type."
        ),
        inputSchema=schema(
            {
                "kind": {
                    "type": "string",
                    "description": "Only this element kind (e.g. Requirement, PhysicalNode)",
                },
                "format": {
                    "type": "string",
                    "enum": list(CATALOGUE_FORMATS),
                    "description": "The whole metamodel as markdown or as JSON",
                    "default": "markdown",
                },
            }
        ),
    ),
    Tool(
        name="arclang_review",
        description=(
            "Ask a judgment model (TypeSafe's Jev) how plausible each trace of a model is "
            "(`arclang review`). Advice only: probabilities, never a verdict, and never part "
            "of the gate. PRIVACY: with `send: true` the kind, name, id, text attributes and "
            "trace rationale of the traced elements go to api.typesafe.ai, and "
            "TYPESAFE_API_KEY must be set. Without it (the default) this is a dry run: it "
            "says what would be sent and sends nothing."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "send": {
                    "type": "boolean",
                    "description": "Really send the questions to api.typesafe.ai",
                    "default": False,
                },
                "suggest": {
                    "type": "boolean",
                    "description": "Also judge undeclared pairs that may be a missing trace",
                    "default": False,
                },
                "threshold": {
                    "type": "number",
                    "description": "Report declared traces under this plausibility (default 0.3)",
                },
                "suggest_threshold": {
                    "type": "number",
                    "description": (
                        "With suggest: report undeclared pairs over this plausibility (default 0.8)"
                    ),
                },
                "max_questions": {
                    "type": "integer",
                    "description": "Most questions asked (default 200)",
                },
            },
            "model_path",
        ),
    ),
]


def check_verdict(run: Run) -> Optional[str]:
    """One line on what `arclang check --lint` found, or None when it did not run through.

    The counts are those of the blocks the compiler printed; a `metamodel:`
    or `constraint:` warning is named as a blocker of the gate.
    """
    if not run.ok or "No compilation errors" not in run.stdout:
        return None
    compilation = parse_block(run.stdout, "Compilation warnings:")
    traceability = parse_block(run.stdout, "Traceability warnings:")
    advisories = parse_block(run.stdout, "methodology advisories:")
    verdict = (
        f"compiles; {len(compilation)} compilation warning(s), "
        f"{len(traceability)} traceability warning(s), {len(advisories)} advisor(ies)"
    )
    if any(line.startswith(("metamodel:", "constraint:")) for line in compilation):
        verdict += "; the metamodel/constraint warnings are blockers of the gate"
    return verdict


def findings_verdict(document: Dict[str, Any]) -> str:
    """One line on the JSON of `arclang check --json`: counts by severity."""
    if not document.get("success", False):
        return f"does not compile; {len(document.get('errors', []))} error(s)"
    counts: Dict[str, int] = {}
    for finding in document.get("findings", []):
        severity = str(finding.get("severity", "?"))
        counts[severity] = counts.get(severity, 0) + 1
    blockers = counts.get(GATE_BLOCKING_SEVERITY, 0)
    others = ", ".join(
        f"{count} {severity}"
        for severity, count in sorted(counts.items())
        if severity != GATE_BLOCKING_SEVERITY
    )
    verdict = f"compiles; {blockers} finding(s) that block the gate"
    return f"{verdict}, {others}" if others else verdict


class CoreTools:
    """Compile, check, rules, trace, gate, diff, impact, export, metamodel, review."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_compile": self._compile,
            "arclang_validate": self._validate,
            "arclang_rules": self._rules,
            "arclang_trace_analysis": self._trace_analysis,
            "arclang_info": self._info,
            "arclang_gate": self._gate,
            "arclang_realize": self._realize,
            "arclang_partition": self._partition,
            "arclang_schedule": self._schedule,
            "arclang_diff": self._diff,
            "arclang_impact": self._impact,
            "arclang_export": self._export,
            "arclang_export_diagram": self._export,
            "arclang_metamodel": self._metamodel,
            "arclang_review": self._review,
            "arclang_cesam": self._cesam,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a core tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown core tool: {tool_name}")
        return await handler(arguments)

    def _model(self, args: Dict[str, Any], key: str = "model_path") -> Path:
        return resolve_path(self.workspace_root, args[key])

    def _line(self, args: Dict[str, Any], *command: str) -> CommandLine:
        return CommandLine(self.workspace_root, args, *command)

    async def _compile(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = self._line(args, "build")
        line.path("output_path", "--output", make_parent=True)
        line.flag("optimize", "--release").value("target", "--target")
        run = await self.compiler.run(line.end(model))
        return report(f"Compilation of {model.name}", run)

    async def _validate(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        title = f"Check of {model.name}"
        line = self._line(args, "check", "--lint").value("method", "--method").flag("safety", "--safety")
        if args.get("format", "text") == "sarif":
            if args.get("as_json", False):
                raise InvalidArguments("'as_json' and 'format: sarif' are two answers: ask for one")
            run = await self.compiler.run(line.add("--format=sarif").end(model))
            if not run.stdout.lstrip().startswith("{"):
                return report(title, run)
            verdict = "SARIF 2.1.0 log" if run.ok else "does not compile; SARIF 2.1.0 log"
            return f"{title}: {verdict}\n\n{run.stdout.rstrip()}"
        if not args.get("as_json", False):
            run = await self.compiler.run(line.end(model))
            return report(title, run, check_verdict(run))
        run = await self.compiler.run(line.add("--json").end(model))
        document = parse_findings(run)
        if document is None:
            return report(title, run)
        # The JSON is the compiler's; only the first line is read from it.
        verdict = findings_verdict(document)
        return f"{title}: {verdict}\n\n{run.stdout.rstrip()}"

    async def _rules(self, args: Dict[str, Any]) -> str:
        wanted = args.get("rule")
        if not wanted:
            run = await self.compiler.rules(as_json=args.get("format", "markdown") == "json")
            return run.stdout if run.ok else report("Validation rules", run)
        run = await self.compiler.rules(as_json=True)
        if not run.ok:
            return report("Validation rules", run)
        rules = {entry["id"]: entry for entry in json.loads(run.stdout) if "id" in entry}
        found = next((rules[name] for name in rules if name.lower() == wanted.lower()), None)
        if found is None:
            return f"No rule '{wanted}'. Known rules: {', '.join(sorted(rules))}"
        return json.dumps(found, indent=2)

    async def _trace_analysis(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        if args.get("review", False):
            return await self._trace_review(model, args)
        if any(args.get(key) for key in ("from", "to", "write")):
            raise InvalidArguments("'from', 'to' and 'write' go with 'review: true'")
        if args.get("suspects", False):
            run = await self.compiler.run(self._line(args, "trace", "--suspects").end(model))
            # The compiler's own first line is the count: it is the verdict.
            verdict = run.stdout.strip().splitlines()[0] if run.ok and run.stdout.strip() else None
            return report(f"Suspect traces of {model.name}", run, verdict)
        run = await self.compiler.run(
            self._line(args, "trace", "--validate").flag("matrix", "--matrix").end(model)
        )
        verdict = None
        if run.ok:
            coverage = parse_coverage(run.stdout)
            issues = parse_block(run.stdout, "issues found:")
            covered = "coverage not printed" if coverage is None else f"coverage {coverage}%"
            verdict = f"{covered}, {len(issues)} issue(s)"
        return report(f"Traceability of {model.name}", run, verdict)

    async def _trace_review(self, model: Path, args: Dict[str, Any]) -> str:
        """`trace --review`: the reviewed source as a diff, written only with `write`."""
        if args.get("suspects", False) or args.get("matrix", False):
            raise InvalidArguments("'review' marks traces as reviewed: ask 'suspects' or 'matrix' apart")
        line = self._line(args, "trace", "--review").value("from", "--from").value("to", "--to")
        title = f"Review of the traces of {model.name}"
        if args.get("write", False):
            run = await self.compiler.run(line.end(model))
            return report(title, run, f"written to {model.name}" if run.ok else None)
        run = await self.compiler.run(line.add("--print").end(model))
        if not run.ok:
            return report(title, run)
        try:
            before = model.read_text()
        except OSError as failure:
            return f"{title}: cannot read {model}: {failure}"
        return diff_against(title, model.name, before, run.stdout)

    async def _info(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        return report(f"Metrics of {model.name}", await self.compiler.info(model))

    async def _gate(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        standard = args.get("standard", "iso26262")
        title = f"Production gate of {model.name}"
        if args.get("format", "text") == "sarif":
            run = await self.compiler.run(
                ["gate", f"--standard={standard}", "--format=sarif", "--", str(model)]
            )
            if not run.stdout.lstrip().startswith("{"):
                return report(title, run)
            # Exit 0 is a gate that passes; the log is printed either way.
            verdict = "PASS; SARIF 2.1.0 log" if run.ok else "FAIL; SARIF 2.1.0 log"
            return f"{title}: {verdict}\n\n{run.stdout.rstrip()}"
        run = await self.compiler.gate(model, standard=standard)
        if run.ok and "\nPRODUCTION GATE: PASS" in run.stdout:
            verdict = "PASS"
        elif not run.ok and "\nPRODUCTION GATE: FAIL" in run.stdout:
            verdict = f"FAIL, {len(parse_blockers(run.stdout))} blocker(s)"
        else:
            # No verdict line: the model did not compile, the gate did not run.
            verdict = None
        return report(title, run, verdict)

    async def _diff(self, args: Dict[str, Any]) -> str:
        two_files = bool(args.get("old_path") or args.get("new_path"))
        revisions = bool(args.get("model_path") or args.get("from_rev") or args.get("to_rev"))
        if two_files == revisions:
            raise InvalidArguments(
                "name the two versions one way: 'old_path' and 'new_path' (two files), or "
                "'model_path' and 'from_rev' (one file between git revisions)"
            )
        if two_files:
            if not (args.get("old_path") and args.get("new_path")):
                raise InvalidArguments("'old_path' and 'new_path' go together")
            old = self._model(args, "old_path")
            new = self._model(args, "new_path")
            title = f"Semantic diff {old.name} -> {new.name}"
            positional = [old, new]
        else:
            if not (args.get("model_path") and args.get("from_rev")):
                raise InvalidArguments("'model_path' and 'from_rev' go together")
            model = self._model(args)
            title = (
                f"Semantic diff of {model.name}: {args['from_rev']} -> "
                f"{args.get('to_rev') or 'the working tree'}"
            )
            positional = [model]
        rich = revisions or bool(args.get("format") or args.get("output_path"))
        if not rich:
            # Two files and no report asked: the JSON of the changes, as before.
            run = await self.compiler.diff(positional[0], positional[1], as_json=True)
            changes = parse_diff(run)
            if changes is None:
                return report(title, run)
            changed = {key: len(value) for key, value in changes.items() if value}
            verdict = (
                ", ".join(f"{count} {key}" for key, count in changed.items())
                if changed
                else "no semantic change"
            )
            return f"{title}: {verdict}\n\n{run.stdout.rstrip()}"
        line = self._line(args, "diff").value("from_rev", "--from").value("to_rev", "--to")
        line.add(f"--format={args.get('format') or 'json'}").value("gate_standard", "--gate-standard")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(*positional))
        # Like diff(1): 0 no difference, 1 a difference; anything else did not run through.
        printed = bool(run.stdout.strip()) or (output is not None and output.is_file())
        if run.returncode == 0:
            verdict = "no semantic change"
        elif run.returncode == 1 and printed:
            verdict = "the versions differ"
        else:
            return report(title, run)
        written = f"\n\nWritten to {output}" if output is not None and output.is_file() else ""
        return f"{title}: {verdict}\n\n{run.output}{written}"

    async def _realize(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        really = args.get("run", False)
        if args.get("keep_going", False) and not really:
            raise InvalidArguments("'keep_going' goes with 'run: true': a plan stops at nothing")
        line = self._line(args, "realize").value("requirement", "--requirement").value("up_to", "--up-to")
        line.flag("board", "--board").add("--json")
        title = f"Realization ladder of {model.name}"
        if not really:
            run = await self.compiler.run(line.add("--dry-run").end(model))
            return report(title, run, "the plan; nothing was run" if run.ok else None)
        limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        run = await self.compiler.run(line.flag("keep_going", "--keep-going").end(model), timeout=limit)
        return report(f"Climb of the realization ladder of {model.name}", run)

    async def _partition(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        really = args.get("run", False)
        if args.get("as_json", False) and not really:
            raise InvalidArguments("'as_json' goes with 'run: true': a plan measures nothing")
        line = self._line(args, "partition").value("target", "--target").value("device", "--device")
        line.value("machine", "--machine")
        if args.get("clock_mhz") is not None:
            clock = args["clock_mhz"]
            if clock <= 0:
                raise InvalidArguments("'clock_mhz' must be positive")
            line.add(f"--clock={clock:g}")
        title = f"HW/SW partition of {model.name}"
        if really:
            limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
            run = await self.compiler.run(line.flag("as_json", "--json").end(model), timeout=limit)
            return report(f"{title} (measured)", run)
        plan = await self.compiler.run(line.add("--dry-run").end(model))
        if not plan.ok:
            return report(title, plan)
        measured = await self.compiler.run(["table", "--kind", "partition", "--format", "md", "--", str(model)])
        answer = report(title, plan, "the two runs; nothing was run")
        return f"{answer}\n\nMeasured so far:\n\n{measured.output.rstrip()}"

    async def _cesam(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        grid_arguments = ["table", "--kind", "cesam_grid", "-f", "json"]
        if args.get("scope"):
            grid_arguments += ["--scope", str(args["scope"])]
        grid = await self.compiler.run([*grid_arguments, "--", str(model)])
        if not grid.ok:
            return report(f"CESAM grid of {model.name}", grid)
        check = await self.compiler.run(["check", "--json", "--method", "cesam", "--", str(model)])
        try:
            table = json.loads(grid.stdout)
            findings = json.loads(check.stdout).get("findings", []) if check.ok else []
        except json.JSONDecodeError as failure:
            return f"CESAM grid of {model.name}: the compiler printed no JSON ({failure})\n\n{grid.output}"
        tables = table if isinstance(table, list) else [table]
        rows = tables[0].get("rows", []) if tables else []
        views = {
            row["id"]: {
                key: {"status": cell.get("status"), "elements": [link["name"] for link in cell.get("links", [])]}
                for key, cell in row.get("cells", {}).items()
                if key not in ("vision", "question")
            }
            for row in rows
        }
        gaps = [issue for row in rows for issue in row.get("issues", [])]
        advisories = [f for f in findings if str(f.get("rule", "")).startswith("cesam.")]
        verdict = "complete" if not gaps and not advisories else f"{len(gaps)} view issue(s), {len(advisories)} advisory(ies)"
        body = json.dumps({"visions": views, "issues": gaps, "advisories": advisories}, indent=2)
        return f"CESAM grid of {model.name}: {verdict}\n\n{body}"

    async def _schedule(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        prove = args.get("prove", False)
        if not prove and (args.get("cycles") is not None or args.get("keep_dir")):
            raise InvalidArguments("'cycles' and 'keep_dir' go with 'prove: true'")
        line = self._line(args, "schedule", "--json").value("target", "--target")
        for machine, micros in (args.get("wcet") or {}).items():
            if micros <= 0:
                raise InvalidArguments(f"the step time of '{machine}' must be positive")
            line.add(f"--wcet={machine}={micros:g}")
        limit = None
        if prove:
            if args.get("cycles") is not None and args["cycles"] < 1:
                raise InvalidArguments("'cycles' must be at least 1")
            line.add("--prove").value("cycles", "--cycles")
            line.path("keep_dir", "--keep")
            limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        run = await self.compiler.run(line.end(model), timeout=limit)
        return report(f"Schedule of {model.name}", run)

    async def _impact(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        seeds = [args[key] for key in ("element", "elements", "change", "delete", "since") if args.get(key)]
        if not seeds:
            raise InvalidArguments(
                "say what changes: 'element', 'elements', 'change', 'delete' or 'since'"
            )
        if args.get("current_path") and not args.get("since"):
            raise InvalidArguments("'current_path' goes with 'since'")
        if args.get("depth") is not None and args["depth"] < 1:
            raise InvalidArguments("'depth' must be at least 1")
        impact_format = args.get("format", "text")
        if impact_format == "xlsx" and not args.get("output_path"):
            raise InvalidArguments("'format: xlsx' is a file: give 'output_path'")
        line = self._line(args, "impact")
        for element in [args.get("element"), *(args.get("elements") or [])]:
            if element:
                line.add(f"--element={element}")
        change = args.get("change")
        if change is not None:
            if not isinstance(change, (dict, list)) or not change:
                raise InvalidArguments("'change' is one edit request (an object) or a list of them")
            # Inline JSON: never `@file` or `-`, which would read a file or the server's input.
            line.add(f"--change={json.dumps(change)}")
        line.value("delete", "--delete").value("since", "--since")
        line.path("current_path", "--current")
        line.value("depth", "--depth").joined("kinds", "--kinds").value("direction", "--direction")
        line.value("gate_standard", "--gate-standard").add(f"--format={impact_format}")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(model))
        subject = (
            f"'{args['element']}'" if args.get("element") and len(seeds) == 1 else "the change"
        )
        answer = report(f"Impact of {subject} in {model.name}", run)
        if run.ok and output is not None and output.is_file():
            answer += f"\n\nWritten to {output}"
        return answer

    async def _export(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        export_format = args.get("format", "html")
        if export_format not in EXPORT_FORMATS:
            return f"Unknown export format '{export_format}'. Known: {', '.join(EXPORT_FORMATS)}"
        requested = args.get("output_path")
        if requested:
            output = resolve_path(self.workspace_root, requested)
            output.parent.mkdir(parents=True, exist_ok=True)
            return await self._export_to(args, model, export_format, output, keep=True)
        # No output_path: nothing of the client's is written, the content is returned.
        with tempfile.TemporaryDirectory(prefix="arclang-export-") as directory:
            output = Path(directory) / f"{model.stem}.{export_format}"
            return await self._export_to(args, model, export_format, output, keep=False)

    async def _export_to(
        self, args: Dict[str, Any], model: Path, export_format: str, output: Path, keep: bool
    ) -> str:
        line = self._line(args, "export", f"--format={export_format}", f"--output={output}")
        line.value("target", "--target").value("binding", "--binding")
        line.flag("split", "--split").flag("no_placement", "--no-placement")
        line.value("strategy", "--strategy").joined("elements", "--element")
        line.value("leaf_functions", "--leaf-functions").value("port_naming", "--port-naming")
        # A directory format compiles (fmu) or writes many files: more than a check takes.
        limit = max(self.compiler.timeout, 120) if export_format in DIRECTORY_EXPORT_FORMATS else None
        run = await self.compiler.run(line.end(model), timeout=limit)
        answer = report(f"Export of {model.name} to {export_format}", run)
        if not run.ok:
            return answer
        if output.is_dir():
            where = f"Written to {output}:" if keep else (
                "Not kept (no output_path: give a directory to keep them); the export is these files:"
            )
            return f"{answer}\n\n{where}\n{listing(output)}"
        if not output.is_file():
            # Files beside the name given (capella, simulink-connector): the compiler names
            # them. Without output_path they were in a directory of the server's own.
            beside = sorted(p.name for p in output.parent.iterdir() if p.is_file())
            if beside and not keep:
                return f"{answer}\n\nNot kept (no output_path); the export is: {', '.join(beside)}"
            return answer
        where = f"Written to {output}" if keep else "Not written (no output_path); the export is"
        try:
            content = output.read_text()
        except UnicodeDecodeError:
            size = output.stat().st_size
            return f"{answer}\n\n{where} binary ({size} bytes, not shown)." if not keep else f"{answer}\n\n{where} (binary, not shown)."
        if len(content) > INLINE_EXPORT_LIMIT:
            return f"{answer}\n\n{where} {len(content)} characters, not shown."
        return f"{answer}\n\n{where}:\n\n{content}"

    async def _metamodel(self, args: Dict[str, Any]) -> str:
        kind = args.get("kind")
        if not kind:
            run = await self.compiler.metamodel(as_json=args.get("format", "markdown") == "json")
            return run.stdout if run.ok else report("Metamodel", run)
        run = await self.compiler.metamodel(as_json=True)
        if not run.ok:
            return report("Metamodel", run)
        metamodel = json.loads(run.stdout)
        kinds = {entry["name"]: entry for entry in metamodel.get("kinds", [])}
        found = next((kinds[name] for name in kinds if name.lower() == kind.lower()), None)
        if found is None:
            return f"No element kind '{kind}'. Known kinds: {', '.join(sorted(kinds))}"
        return json.dumps(found, indent=2)

    async def _review(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        send = args.get("send", False)
        run = await self.compiler.review(
            model,
            send=send,
            suggest=args.get("suggest", False),
            threshold=args.get("threshold"),
            suggest_threshold=args.get("suggest_threshold"),
            max_questions=args.get("max_questions"),
        )
        verdict = None
        if run.ok and not send:
            verdict = "dry run, nothing was sent"
        elif run.ok and "Nothing to review" in run.output:
            verdict = "nothing to review, nothing was sent"
        elif run.ok:
            verdict = "advice received (probabilities, not a verdict)"
        return report(f"Trace review of {model.name}", run, verdict)
