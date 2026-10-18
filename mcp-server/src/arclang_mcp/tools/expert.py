"""
MBSE helper tools: keyword-based requirement sorting and SVG diagram checks.

These tools do not judge a model. They sort sentences by the Arcadia layer
their wording suggests and look for marks in diagram files. The model
written from an analysis is compiled before it is returned.
"""

import json
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import DIAGRAM_TYPES, ArcLangCompiler
from ..mbse_expert import MBSECapellaExpert
from .batch import ENHANCEMENTS
from .common import resolve_path, schema
from .generation import has_warnings

TOOLS: List[Tool] = [
    Tool(
        name="arclang_analyze_requirements",
        description=(
            "Sort natural-language requirements by Arcadia layer (operational, system, "
            "logical, physical) from the words they use, pick out those that mention "
            "safety with the level they STATE (none is assumed), and list component hints "
            "(sensor, controller, actuator). Keyword matching, not understanding: review "
            "the result. Returns JSON for arclang_generate_from_requirements."
        ),
        inputSchema=schema(
            {
                "requirements_text": {
                    "type": "string",
                    "description": "Natural language requirements, one per line",
                },
                "domain": {
                    "type": "string",
                    "enum": ["automotive", "aerospace", "defense", "industrial", "general"],
                    "description": "Application domain (recorded in the analysis)",
                    "default": "automotive",
                },
            },
            "requirements_text",
        ),
    ),
    Tool(
        name="arclang_generate_from_requirements",
        description=(
            "Write an ArcLang skeleton from an analysis: the requirements as "
            "stakeholder/system/safety requirements and one logical component per component "
            "hint. It invents no trace, no actor and no hardware. The skeleton is compiled "
            "before it is returned, and the answer says whether the compiler accepted it."
        ),
        inputSchema=schema(
            {
                "analysis": {
                    "type": "object",
                    "description": "Analysis from arclang_analyze_requirements",
                },
                "project_name": {"type": "string", "description": "Name of the model"},
                "output_path": {
                    "type": "string",
                    "description": "Where to save the .arc file (optional)",
                },
            },
            "analysis",
            "project_name",
        ),
    ),
    Tool(
        name="arclang_assess_diagram_quality",
        description=(
            "Look for expected marks in an SVG diagram file (interface symbols, protocol "
            "labels, safety borders, nesting). String checks on the SVG text, implemented "
            "for component, physical and operational diagrams only; for another type the "
            "tool says it has no check instead of giving a score."
        ),
        inputSchema=schema(
            {
                "diagram_path": {"type": "string", "description": "Path to the SVG file"},
                "diagram_type": {
                    "type": "string",
                    "enum": list(DIAGRAM_TYPES),
                    "description": "Type of diagram",
                },
            },
            "diagram_path",
            "diagram_type",
        ),
    ),
    Tool(
        name="arclang_suggest_diagram_enhancements",
        description="Turn the failed checks of arclang_assess_diagram_quality into a list of "
        "enhancement names for the `enhance_diagram` batch action.",
        inputSchema=schema(
            {
                "assessment": {
                    "type": "object",
                    "description": "Assessment from arclang_assess_diagram_quality",
                },
            },
            "assessment",
        ),
    ),
    Tool(
        name="arclang_generate_mbse_report",
        description=(
            "Markdown summary of a requirement analysis and of diagram assessments: counts "
            "per layer, safety requirements with their stated level, diagram check results."
        ),
        inputSchema=schema(
            {
                "analysis": {"type": "object", "description": "Requirement analysis"},
                "assessments": {
                    "type": "array",
                    "items": {"type": "object"},
                    "description": "Diagram assessments",
                },
                "output_path": {"type": "string", "description": "Where to save the report"},
            },
            "analysis",
        ),
    ),
]


class ExpertTools:
    """Requirement sorting, skeleton writing and diagram checks."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self.expert = MBSECapellaExpert(workspace_root)
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_analyze_requirements": self._analyze_requirements,
            "arclang_generate_from_requirements": self._generate_from_requirements,
            "arclang_assess_diagram_quality": self._assess_diagram_quality,
            "arclang_suggest_diagram_enhancements": self._suggest_diagram_enhancements,
            "arclang_generate_mbse_report": self._generate_mbse_report,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute an MBSE helper tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown MBSE helper tool: {tool_name}")
        return await handler(arguments)

    async def _analyze_requirements(self, args: Dict[str, Any]) -> str:
        analysis = self.expert.analyze_requirements(args.get("requirements_text", ""))
        analysis["domain"] = args.get("domain", "automotive")
        analysis["method"] = "keyword matching; review before use"
        return json.dumps(analysis, indent=2)

    async def _generate_from_requirements(self, args: Dict[str, Any]) -> str:
        content = self.expert.generate_arclang_skeleton(
            args.get("analysis", {}), args.get("project_name", "GeneratedModel")
        )
        with tempfile.TemporaryDirectory(prefix="arclang-skeleton-") as directory:
            candidate = Path(directory) / "skeleton.arc"
            candidate.write_text(content)
            run = await self.compiler.check(candidate)
            refusal = run.output.replace(str(candidate), "the skeleton")
        if not run.ok:
            return (
                "THE COMPILER REFUSES THIS SKELETON; it was not saved.\n\n"
                f"{refusal}\n\n```arc\n{content}```"
            )
        if has_warnings(run.output):
            return (
                "THE COMPILER WARNS ABOUT THIS SKELETON (a `metamodel:` warning blocks "
                f"arclang_gate); it was not saved.\n\n{refusal}\n\n```arc\n{content}```"
            )
        saved = "Not saved (no output_path)."
        output_path = args.get("output_path")
        if output_path:
            target = resolve_path(self.workspace_root, output_path)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)
            saved = f"Saved to {target}."
        return (
            f"The compiler accepts this skeleton. {saved} It has no trace yet: "
            f"arclang_gate will fail until the requirements are satisfied and verified.\n\n"
            f"```arc\n{content}```"
        )

    async def _assess_diagram_quality(self, args: Dict[str, Any]) -> str:
        diagram = resolve_path(self.workspace_root, args.get("diagram_path", ""))
        assessment = self.expert.assess_diagram_quality(
            diagram, args.get("diagram_type", "component")
        )
        return json.dumps(assessment, indent=2)

    async def _suggest_diagram_enhancements(self, args: Dict[str, Any]) -> str:
        assessment = args.get("assessment", {})
        if not assessment.get("assessed", True) or "error" in assessment:
            return json.dumps(
                {"suggested_enhancements": [], "reason": "the diagram was not assessed"}
            )
        suggested = self.expert.suggest_enhancements(assessment)
        return json.dumps(
            {
                "suggested_enhancements": [name for name in suggested if name in ENHANCEMENTS],
                "not_implemented_by_enhance_diagram": [
                    name for name in suggested if name not in ENHANCEMENTS
                ],
            },
            indent=2,
        )

    async def _generate_mbse_report(self, args: Dict[str, Any]) -> str:
        report = self.expert.generate_mbse_report(
            args.get("analysis", {}), args.get("assessments", [])
        )
        output_path = args.get("output_path")
        if not output_path:
            return report
        target = resolve_path(self.workspace_root, output_path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(report)
        return f"Report saved to {target}\n\n{report}"
