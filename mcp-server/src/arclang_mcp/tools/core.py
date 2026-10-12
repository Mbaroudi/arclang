"""
Core tools: each one is a subcommand of the compiler and returns what it printed.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import (
    EXPORT_FORMATS,
    SAFETY_STANDARDS,
    ArcLangCompiler,
    parse_blockers,
    parse_block,
    parse_coverage,
    parse_diff,
)
from .common import MODEL_PATH, report, resolve_path, schema

# Formats whose result is short enough to hand back to the caller in full.
INLINE_EXPORT_LIMIT = 20_000

STANDARD = {
    "type": "string",
    "enum": list(SAFETY_STANDARDS),
    "description": "Safety standard",
    "default": "iso26262",
}

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
            "Arcadia methodology advisories. Run it after every edit. A requirement or "
            "component is listed as having no trace only when no trace reaches it in "
            "either direction; a trace on a function covers its component."
        ),
        inputSchema=schema({"model_path": MODEL_PATH}, "model_path"),
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
            "Production-readiness gate (`arclang gate`): PASS or FAIL against what an "
            "industrial design review requires (every requirement satisfied and verified "
            "by a test_case, HARA and ASIL consistency, typed attributes, constraints). "
            "Returns the verdict and each blocker with its rule. A FAIL is a result, "
            "not a tool error."
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
            "Export a model (`arclang export`). `sys-ml` is the SysML v2 textual notation, "
            "`sysml-json` its abstract syntax as KerML/SysML records, `capella` the Capella "
            "XML, `req-if` the requirements, `c-headers` and `proto` the interface "
            "contracts. Short results are returned; longer ones are only written to the file."
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
                "max_questions": {
                    "type": "integer",
                    "description": "Most questions asked (default 200)",
                },
            },
            "model_path",
        ),
    ),
]


class CoreTools:
    """Compile, check, trace, gate, diff, impact, export, metamodel, review."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_compile": self._compile,
            "arclang_validate": self._validate,
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
        run = await self.compiler.check(model)
        verdict = None
        if run.ok and "No compilation errors" in run.stdout:
            compilation = parse_block(run.stdout, "Compilation warnings:")
            traceability = parse_block(run.stdout, "Traceability warnings:")
            advisories = parse_block(run.stdout, "methodology advisories:")
            verdict = (
                f"compiles; {len(compilation)} compilation warning(s), "
                f"{len(traceability)} traceability warning(s), {len(advisories)} advisor(ies)"
            )
            if any(line.startswith(("metamodel:", "constraint:")) for line in compilation):
                verdict += "; the metamodel/constraint warnings are blockers of the gate"
        return report(f"Check of {model.name}", run, verdict)

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
        run = await self.compiler.gate(model, standard=args.get("standard", "iso26262"))
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
