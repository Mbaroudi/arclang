"""
Core tools: each one is a subcommand of the compiler and returns what it printed.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..compiler.wrapper import (
    CATALOGUE_FORMATS,
    EXPORT_FORMATS,
    SAFETY_STANDARDS,
    ArcLangCompiler,
    Run,
    parse_block,
    parse_blockers,
    parse_coverage,
    parse_diff,
    parse_findings,
)
from .common import MODEL_PATH, report, resolve_path, schema

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
            "either direction; a trace on a function covers its component."
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
            "and the trace matrix with each rationale when `matrix` is true."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "matrix": {
                    "type": "boolean",
                    "description": "Also print the traceability matrix",
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
            "rule. A FAIL is a result, not a tool error."
        ),
        inputSchema=schema({"model_path": MODEL_PATH, "standard": STANDARD}, "model_path"),
    ),
    Tool(
        name="arclang_diff",
        description=(
            "Semantic diff between two versions of a model (`arclang diff --json`). "
            "Elements are compared by stable identity: moving a block is no change, "
            "renaming an element is a modification, not a removal and an addition. "
            "It reports elements, traces and other relationships added, removed or "
            "modified, with each changed attribute (old and new value). A reworded "
            "trace rationale is a change. Element types are metamodel kinds."
        ),
        inputSchema=schema(
            {
                "old_path": {"type": "string", "description": "The old version of the model"},
                "new_path": {"type": "string", "description": "The new version of the model"},
            },
            "old_path",
            "new_path",
        ),
    ),
    Tool(
        name="arclang_impact",
        description=(
            "Change-impact analysis (`arclang impact`): everything transitively affected "
            "when one element changes, by distance, with the relation that carries the impact."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": {
                    "type": "string",
                    "description": 'Element id or unambiguous name, e.g. "REQ-AEB-004"',
                },
            },
            "model_path",
            "element",
        ),
    ),
    Tool(
        name="arclang_export",
        description=(
            "Export a model (`arclang export -f <format>`). `sys-ml` is the SysML v2 textual "
            "notation, `sysml-json` its abstract syntax as the JSON records of the Systems "
            "Modeling API, `capella` the Capella project (.capella, .aird, .afm), `req-if` "
            "the requirements, `c-headers` and `proto` the interface contracts, `json` the "
            "compiled model, `markdown`, `mermaid`, `plant-uml`, `html` documents. Short "
            "results are returned; longer ones are only written to the file. To read a "
            "SysML, Capella or ReqIF file into ArcLang use arclang_import."
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
                    "description": "Output file (default: a temporary file)",
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
            "arclang_diff": self._diff,
            "arclang_impact": self._impact,
            "arclang_export": self._export,
            "arclang_export_diagram": self._export,
            "arclang_metamodel": self._metamodel,
            "arclang_review": self._review,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a core tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown core tool: {tool_name}")
        return await handler(arguments)

    def _model(self, args: Dict[str, Any], key: str = "model_path") -> Path:
        return resolve_path(self.workspace_root, args[key])

    async def _compile(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        requested = args.get("output_path")
        output = resolve_path(self.workspace_root, requested) if requested else None
        if output:
            output.parent.mkdir(parents=True, exist_ok=True)
        run = await self.compiler.build(
            model,
            output=output,
            release=args.get("optimize", False),
        )
        return report(f"Compilation of {model.name}", run)

    async def _validate(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        title = f"Check of {model.name}"
        if not args.get("as_json", False):
            run = await self.compiler.check(model)
            return report(title, run, check_verdict(run))
        run = await self.compiler.check(model, as_json=True)
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
        run = await self.compiler.trace(model, matrix=args.get("matrix", False))
        verdict = None
        if run.ok:
            coverage = parse_coverage(run.stdout)
            issues = parse_block(run.stdout, "issues found:")
            covered = "coverage not printed" if coverage is None else f"coverage {coverage}%"
            verdict = f"{covered}, {len(issues)} issue(s)"
        return report(f"Traceability of {model.name}", run, verdict)

    async def _info(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        return report(f"Metrics of {model.name}", await self.compiler.info(model))

    async def _gate(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        standard = args.get("standard", "iso26262")
        run = await self.compiler.gate(model, standard=standard)
        if run.ok and "\nPRODUCTION GATE: PASS" in run.stdout:
            verdict = "PASS"
        elif not run.ok and "\nPRODUCTION GATE: FAIL" in run.stdout:
            verdict = f"FAIL, {len(parse_blockers(run.stdout))} blocker(s)"
        else:
            # No verdict line: the model did not compile, the gate did not run.
            verdict = None
        return report(f"Production gate of {model.name}", run, verdict)

    async def _diff(self, args: Dict[str, Any]) -> str:
        old = self._model(args, "old_path")
        new = self._model(args, "new_path")
        run = await self.compiler.diff(old, new, as_json=True)
        title = f"Semantic diff {old.name} -> {new.name}"
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

    async def _impact(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        element = args["element"]
        return report(
            f"Impact of '{element}' in {model.name}", await self.compiler.impact(model, element)
        )

    async def _export(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        export_format = args.get("format", "html")
        if export_format not in EXPORT_FORMATS:
            return f"Unknown export format '{export_format}'. Known: {', '.join(EXPORT_FORMATS)}"
        requested = args.get("output_path")
        if requested:
            output = resolve_path(self.workspace_root, requested)
            output.parent.mkdir(parents=True, exist_ok=True)
            return await self._export_to(model, export_format, output, keep=True)
        # No output_path: nothing of the client's is written, the content is returned.
        with tempfile.TemporaryDirectory(prefix="arclang-export-") as directory:
            output = Path(directory) / f"{model.stem}.{export_format}"
            return await self._export_to(model, export_format, output, keep=False)

    async def _export_to(self, model: Path, export_format: str, output: Path, keep: bool) -> str:
        run = await self.compiler.export(model, export_format, output)
        answer = report(f"Export of {model.name} to {export_format}", run)
        if not run.ok or not output.is_file():
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
            run = await self.compiler.metamodel(as_json=False)
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
