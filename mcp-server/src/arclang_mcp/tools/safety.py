"""
Safety tools: what the compiler reports about the safety content of a model.

Neither tool certifies anything. `arclang_safety_check` returns the output
of `arclang safety`: the counts of declared safety content, and with `fmea`,
`fta` or `report` the FMEA entries, the fault tree of every hazard, or the
safety report in Markdown, each computed from the model. The verdict on a
model is `arclang_gate`'s.
`arclang_hazard_analysis` lists the hazards the model DECLARES: it derives
no hazard of its own.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import SAFETY_STANDARDS, ArcLangCompiler
from .common import MODEL_PATH, CommandLine, report, resolve_path, schema

TOOLS: List[Tool] = [
    Tool(
        name="arclang_safety_check",
        description=(
            "Safety content of a model (`arclang safety`): safety analysis blocks, hazards "
            "and FMEA entries counted for a standard. `fmea: true` reports the FMEA entries "
            "(`--fmea`), `fta: true` the fault tree of every hazard with its failure modes, "
            "rate and budget (`--fta`), `report: true` the safety report in Markdown "
            "(hazards, FMEA by risk, fault trees, mitigations: `--report`), written to "
            "`output_path` when given. Every figure is computed from the model. This is not "
            "a compliance verdict: for PASS/FAIL (HARA and ASIL consistency, mitigations) "
            "use arclang_gate."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "standard": {
                    "type": "string",
                    "enum": list(SAFETY_STANDARDS),
                    "description": "Safety standard: iso26262, do178c or iec61508",
                },
                "fmea": {"type": "boolean", "description": "Report the FMEA entries", "default": False},
                "fta": {
                    "type": "boolean",
                    "description": "The fault tree of every hazard",
                    "default": False,
                },
                "report": {
                    "type": "boolean",
                    "description": "The safety report in Markdown",
                    "default": False,
                },
                "output_path": {
                    "type": "string",
                    "description": "Where the report is written (default: returned)",
                },
            },
            "model_path",
            "standard",
        ),
    ),
    Tool(
        name="arclang_hazard_analysis",
        description=(
            "List the hazards and FMEA entries a model declares in its `safety_analysis` "
            "blocks, with their attributes (severity, exposure, controllability, ASIL, "
            "mitigated_by). It reads the model; it does not identify hazards for you."
        ),
        inputSchema=schema({"model_path": MODEL_PATH}, "model_path"),
    ),
]


class SafetyTools:
    """Safety summary and declared hazards."""

    def __init__(self, compiler: ArcLangCompiler, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config
        self.workspace_root = compiler.workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_safety_check": self._safety_check,
            "arclang_hazard_analysis": self._hazard_analysis,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a safety tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown safety tool: {tool_name}")
        return await handler(arguments)

    async def _safety_check(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        standard = args["standard"]
        line = CommandLine(self.workspace_root, args, "safety", f"--standard={standard}")
        line.flag("fmea", "--fmea").flag("fta", "--fta").flag("report", "--report")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(model))
        asked = [name for name in ("fmea", "fta", "report") if args.get(name)]
        what = ", ".join(asked) if asked else "summary"
        verdict = f"{what} printed (not a compliance verdict)" if run.ok else None
        answer = report(f"Safety content of {model.name} ({standard})", run, verdict)
        if run.ok and output is not None and output.is_file():
            answer += f"\n\nWritten to {output}"
        return answer

    async def _hazard_analysis(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        with tempfile.TemporaryDirectory(prefix="arclang-hazards-") as directory:
            exported = Path(directory) / "model.json"
            run = await self.compiler.export(model, "json", exported)
            if not run.ok or not exported.is_file():
                return report(f"Hazards of {model.name}", run)
            blocks = json.loads(exported.read_text()).get("safety_analysis", [])
        declared = {
            "hazards": [hazard for block in blocks for hazard in block.get("hazards", [])],
            "fmea": [entry for block in blocks for entry in block.get("fmea", [])],
        }
        count = f"{len(declared['hazards'])} hazard(s), {len(declared['fmea'])} FMEA entr(ies)"
        return f"Declared in {model.name}: {count}\n\n{json.dumps(declared, indent=2)}"
