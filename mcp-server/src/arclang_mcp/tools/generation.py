"""
Generation tools: ArcLang snippets from a description, and diagrams.

A generated snippet is compiled before it is returned, inside the block it
belongs to, and the answer says whether the compiler accepted it. A snippet
the compiler refuses is still returned, with the compiler's error, and is
never presented as valid.
"""

import re
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..ai.generator import COMPONENT_BLOCKS, AIGenerator
from ..compiler.wrapper import (
    DIAGRAM_TYPES,
    TEXT_DIAGRAM_FORMATS,
    VIEWPOINT_FORMATS,
    VIEWS,
    ArcLangCompiler,
    Run,
)
from .common import MODEL_PATH, report, resolve_path, schema

VIEWPOINT_EXTENSIONS = {"viewpoints": "json", "elk": "json", "html": "html"}
# File extension of each `arclang diagram -f` format arclang_generate_diagram offers.
DIAGRAM_EXTENSIONS = {
    **{name: "svg" for name in DIAGRAM_TYPES},
    "mermaid": "mmd",
    "plant-uml": "puml",
}

TOOLS: List[Tool] = [
    Tool(
        name="arclang_generate_requirement",
        description=(
            "Write an ArcLang requirement from a description, and compile it. Returns the "
            '`req "ID" "Title" { ... }` declaration, to be placed in a '
            "`requirements <kind> { }` block, and whether the compiler accepted it. "
            "Uses a template, or a Claude model when ANTHROPIC_API_KEY is set."
        ),
        inputSchema=schema(
            {
                "description": {
                    "type": "string",
                    "description": "What the system shall do, in natural language",
                },
                "safety_level": {
                    "type": "string",
                    "description": 'Safety level as the metamodel spells it, e.g. "ASIL-B", "DAL-A", "SIL-2"',
                },
                "priority": {
                    "type": "string",
                    "enum": ["Critical", "High", "Medium", "Low"],
                    "description": "Requirement priority",
                    "default": "High",
                },
            },
            "description",
        ),
    ),
    Tool(
        name="arclang_generate_component",
        description=(
            "Write an ArcLang logical component or physical node from a description, and "
            "compile it. Returns the declaration, to be placed in `architecture logical { }` "
            "or `architecture physical { }`, and whether the compiler accepted it."
        ),
        inputSchema=schema(
            {
                "description": {
                    "type": "string",
                    "description": "What the component is or does",
                },
                "component_type": {
                    "type": "string",
                    "enum": sorted(COMPONENT_BLOCKS),
                    "description": "Logical component, or Physical node",
                    "default": "Logical",
                },
                "safety_level": {"type": "string", "description": "Safety level (optional)"},
            },
            "description",
        ),
    ),
    Tool(
        name="arclang_suggest_architecture",
        description=(
            "Ask a Claude model for components that would meet a list of requirements. "
            "Needs ANTHROPIC_API_KEY; without it the tool says so and suggests nothing. "
            "The answer is a suggestion, not a model: nothing is compiled."
        ),
        inputSchema=schema(
            {
                "requirements": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Requirement descriptions",
                },
                "domain": {
                    "type": "string",
                    "enum": ["automotive", "aerospace", "defense", "industrial"],
                    "description": "Application domain",
                },
            },
            "requirements",
        ),
    ),
    Tool(
        name="arclang_viewpoint_diagrams",
        description=(
            "The Arcadia viewpoint diagrams of a model (`arclang diagram`): operational "
            "(oab), system (sab), logical (lab), physical (pab), modes and states (msm), "
            "scenarios (es). `html` is a self-contained viewer; `viewpoints` is the diagram "
            "model as JSON; `elk` the ELK layout graphs. Returns what was written and where."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "format": {
                    "type": "string",
                    "enum": list(VIEWPOINT_FORMATS),
                    "description": "Output format",
                    "default": "html",
                },
                "view": {
                    "type": "string",
                    "enum": list(VIEWS),
                    "description": "Only this kind of view (default: all)",
                },
                "output_path": {
                    "type": "string",
                    "description": "Output file (default: next to the model)",
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_generate_diagram",
        description=(
            "One diagram of a model (`arclang diagram -f <type>`): an SVG for the drawn "
            "types (they need the Node renderer, ARCLANG_DIAGRAM_SERVICE), a text diagram "
            "for `mermaid` (.mmd) and `plant-uml` (.puml). A type the model has no content "
            "for (no scenario, no state machine) fails: the compiler's message is returned. "
            "For the Arcadia viewpoints prefer arclang_viewpoint_diagrams."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "diagram_type": {
                    "type": "string",
                    "enum": [*DIAGRAM_TYPES, *TEXT_DIAGRAM_FORMATS],
                    "description": "Type of diagram",
                },
                "output_path": {
                    "type": "string",
                    "description": (
                        "Output file (default: <type>.svg, .mmd or .puml in the workspace)"
                    ),
                },
            },
            "model_path",
            "diagram_type",
        ),
    ),
    Tool(
        name="arclang_generate_all_diagrams",
        description=(
            "Every SVG diagram type of a model, one file per type in a directory. Types "
            "the model has no content for are reported as not generated, with the reason."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "output_dir": {
                    "type": "string",
                    "description": "Output directory",
                    "default": "./diagrams",
                },
            },
            "model_path",
        ),
    ),
]


def has_warnings(output: str) -> bool:
    """Whether `arclang check` printed a compilation warning (`metamodel:`, `constraint:`...).

    `check` exits 0 with them, but a `metamodel:` warning is a gate blocker,
    so a snippet that gets one is not "accepted".
    """
    return "Compilation warnings:" in output


def is_one_block(snippet: str) -> bool:
    """True when braces balance and never close more than the snippet opened.

    Strings and comments are skipped, so a brace inside them does not count.
    """
    code = re.sub(r'"(?:\\.|[^"\\])*"|//[^\n]*|/\*.*?\*/', "", snippet, flags=re.DOTALL)
    depth = 0
    for character in code:
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth < 0:
                return False
    return depth == 0


def failure_reason(run: Run) -> str:
    """Why a command failed: its `Error` line, else the first thing it said."""
    said = (run.stderr + "\n" + run.stdout).splitlines()
    lines = [line.strip() for line in said if line.strip()]
    errors = [line for line in lines if line.startswith(("Error", "error", "✗"))]
    return (errors or lines or [f"exit {run.returncode}"])[0]


class GenerationTools:
    """Snippets and diagrams."""

    def __init__(self, compiler: ArcLangCompiler, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config
        self.workspace_root = compiler.workspace_root
        self.generator = AIGenerator(config.get("ai", {}))
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_generate_requirement": self._generate_requirement,
            "arclang_generate_component": self._generate_component,
            "arclang_suggest_architecture": self._suggest_architecture,
            "arclang_viewpoint_diagrams": self._viewpoint_diagrams,
            "arclang_generate_diagram": self._generate_diagram,
            "arclang_generate_all_diagrams": self._generate_all_diagrams,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a generation tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown generation tool: {tool_name}")
        return await handler(arguments)

    async def compile_snippet(self, snippet: str, block: str) -> Optional[str]:
        """Compile `snippet` inside `block`. None when accepted, else the compiler's words."""
        if not snippet.strip():
            return "the snippet is empty"
        if not is_one_block(snippet):
            return (
                "the snippet's braces do not balance on their own: it would close the block "
                "it is placed in"
            )
        with tempfile.TemporaryDirectory(prefix="arclang-snippet-") as directory:
            model = Path(directory) / "snippet.arc"
            model.write_text(f"model Snippet {{\n{block} {{\n{snippet}\n}}\n}}\n")
            run = await self.compiler.check(model)
        if run.ok and not has_warnings(run.output):
            return None
        return run.output.replace(f"{model}: ", "").replace(str(model), "the snippet")

    async def _snippet_answer(self, snippet: str, block: str) -> str:
        source = "a Claude model" if self.generator.uses_model else "the built-in template"
        refusal = await self.compile_snippet(snippet, block)
        if refusal is None:
            status = f"The compiler accepts it inside `{block} {{ }}` without a warning."
        elif "No compilation errors" in refusal:
            status = (
                f"THE COMPILER WARNS ABOUT IT inside `{block} {{ }}`; a `metamodel:` warning "
                f"blocks arclang_gate. Fix it before use.\n\n{refusal}"
            )
        else:
            status = (
                f"THE COMPILER REFUSES IT inside `{block} {{ }}`; fix it before use. "
                f"Line numbers count from the snippet's first line, plus 2.\n\n{refusal}"
            )
        return f"Written by {source}.\n\n```arc\n{snippet}\n```\n\n{status}"

    async def _generate_requirement(self, args: Dict[str, Any]) -> str:
        snippet = await self.generator.generate_requirement(
            description=args["description"],
            safety_level=args.get("safety_level"),
            priority=args.get("priority", "High"),
        )
        return await self._snippet_answer(snippet, "requirements system")

    async def _generate_component(self, args: Dict[str, Any]) -> str:
        component_type = args.get("component_type", "Logical")
        if component_type not in COMPONENT_BLOCKS:
            return (
                f"Unknown component type '{component_type}'. "
                f"Known: {', '.join(sorted(COMPONENT_BLOCKS))}"
            )
        snippet = await self.generator.generate_component(
            description=args["description"],
            component_type=component_type,
            safety_level=args.get("safety_level"),
        )
        return await self._snippet_answer(snippet, COMPONENT_BLOCKS[component_type][0])

    async def _suggest_architecture(self, args: Dict[str, Any]) -> str:
        requirements = args["requirements"]
        suggestions = await self.generator.suggest_architecture(
            requirements=requirements, domain=args.get("domain", "automotive")
        )
        if suggestions is None:
            return (
                "No suggestion: this tool asks a Claude model and ANTHROPIC_API_KEY is not "
                "set. Nothing was made up in its place."
            )
        lines = [
            f"Suggested by a Claude model from {len(requirements)} requirement(s); not compiled."
        ]
        components = suggestions.get("components", [])
        for component in components if isinstance(components, list) else []:
            if not isinstance(component, dict):
                lines.append(f"- {component!r} (not the shape asked for)")
                continue
            level = f", {component['safety_level']}" if component.get("safety_level") else ""
            lines.append(
                f"- {component.get('name', '?')} ({component.get('type', '?')}{level}): "
                f"{component.get('description', '')}"
            )
        if suggestions.get("patterns"):
            lines.append("Patterns: " + ", ".join(suggestions["patterns"]))
        if suggestions.get("unparsed_answer"):
            lines.append(
                "The answer was not the JSON asked for:\n" + suggestions["unparsed_answer"]
            )
        return "\n".join(lines)

    async def _viewpoint_diagrams(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        diagram_format = args.get("format", "html")
        if diagram_format not in VIEWPOINT_FORMATS:
            return f"Unknown format '{diagram_format}'. Known: {', '.join(VIEWPOINT_FORMATS)}"
        requested = args.get("output_path")
        if requested:
            output = resolve_path(self.workspace_root, requested)
        else:
            extension = VIEWPOINT_EXTENSIONS[diagram_format]
            output = model.with_name(f"{model.stem}.{diagram_format}.{extension}")
        output.parent.mkdir(parents=True, exist_ok=True)
        run = await self.compiler.diagram(model, diagram_format, output, view=args.get("view"))
        return report(f"Viewpoint diagrams of {model.name} ({diagram_format})", run)

    async def _generate_diagram(self, args: Dict[str, Any]) -> str:
        diagram_type = args["diagram_type"]
        extension = DIAGRAM_EXTENSIONS.get(diagram_type)
        if extension is None:
            return f"Unknown diagram type '{diagram_type}'. Known: {', '.join(DIAGRAM_EXTENSIONS)}"
        model = resolve_path(self.workspace_root, args["model_path"])
        output = resolve_path(
            self.workspace_root, args.get("output_path") or f"{diagram_type}.{extension}"
        )
        output.parent.mkdir(parents=True, exist_ok=True)
        run = await self.compiler.diagram(model, diagram_type, output)
        return report(f"{diagram_type} diagram of {model.name}", run)

    async def generate_all(self, model: Path, output_dir: Path) -> Dict[str, Dict[str, Any]]:
        """Each diagram type: the file written and its size, or why there is none."""
        output_dir.mkdir(parents=True, exist_ok=True)
        results: Dict[str, Dict[str, Any]] = {}
        for diagram_type in DIAGRAM_TYPES:
            output = output_dir / f"{diagram_type}.svg"
            run = await self.compiler.diagram(model, diagram_type, output)
            if run.ok and output.is_file():
                results[diagram_type] = {"path": str(output), "bytes": output.stat().st_size}
            else:
                results[diagram_type] = {"error": failure_reason(run)}
        return results

    async def _generate_all_diagrams(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        output_dir = resolve_path(self.workspace_root, args.get("output_dir", "./diagrams"))
        results = await self.generate_all(model, output_dir)
        written = [name for name, result in results.items() if "path" in result]
        lines = [
            f"Diagrams of {model.name}: {len(written)}/{len(results)} generated in {output_dir}",
            "",
        ]
        for name, result in results.items():
            if "path" in result:
                lines.append(f"- {name}: {result['path']} ({result['bytes']} bytes)")
            else:
                lines.append(f"- {name}: NOT generated: {result['error']}")
        return "\n".join(lines)
