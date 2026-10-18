"""
Safety tools: what the compiler reports about the safety content of a model.

Neither tool certifies anything. `arclang_safety_check` returns the output
of `arclang safety`; the verdict on a model is `arclang_gate`'s.
`arclang_hazard_analysis` lists the hazards the model DECLARES: it derives
no hazard of its own.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import SAFETY_STANDARDS, ArcLangCompiler
from .common import MODEL_PATH, report, resolve_path, schema

TOOLS: List[Tool] = [
    Tool(
        name="arclang_safety_check",
        description=(
            "Safety content of a model (`arclang safety`): safety analysis blocks, hazards "
            "and FMEA entries counted for a standard. This is a summary, not a compliance "
            "verdict: for PASS/FAIL (HARA and ASIL consistency, mitigations) use arclang_gate."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "standard": {
                    "type": "string",
                    "enum": list(SAFETY_STANDARDS),
                    "description": "Safety standard",
                },
                "generate_report": {
                    "type": "boolean",
                    "description": "Pass --report",
                    "default": False,
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
        run = await self.compiler.safety(
            model, standard=standard, report=args.get("generate_report", False)
        )
        verdict = "summary printed (not a compliance verdict)" if run.ok else None
        return report(f"Safety content of {model.name} ({standard})", run, verdict)

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
