"""
Batch tool: several operations in one call, each reduced to what matters.

Every figure returned here is read from the compiler's output or from a
file on disk. An operation whose command fails is reported as an error
with the compiler's message; none is counted as a success on the strength
of the words it printed.
"""

import json
import re
import tempfile
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import (
    ArcLangCompiler,
    Run,
    parse_block,
    parse_blockers,
    parse_coverage,
    parse_diff,
    parse_metrics,
)
from .common import resolve_path, schema
from .generation import GenerationTools

# SVG rewrites `enhance_diagram` can apply.
# `increase_spacing` is not offered: scaling the viewBox zooms the drawing out, it
# does not space the elements. Asking for it gets "not implemented".
ENHANCEMENTS = ("improve_contrast", "add_grid", "enlarge_text")

GRID_PATTERN = (
    '<pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">'
    '<path d="M 20 0 L 0 0 0 20" fill="none" stroke="#E0E0E0" stroke-width="0.5"/>'
    "</pattern>"
)
SCALE = 1.2
DARKEN = 0.8
LIGHT_COLOUR_SUM = 500

ACTIONS = (
    "compile_and_validate",
    "gate",
    "generate_all_diagrams",
    "analyze_traceability",
    "safety_check",
    "extract_metrics",
    "export_filtered",
    "compare_models",
    "analyze_diagrams",
    "enhance_diagram",
)

TOOLS: List[Tool] = [
    Tool(
        name="arclang_execute_batch",
        description=(
            "Run several operations in one call and get a reduced result for each, read "
            "from the compiler's output.\n"
            "- compile_and_validate {model_path}: errors, warnings, counts\n"
            "- gate {model_path, standard?}: verdict and blockers\n"
            "- generate_all_diagrams {model_path, output_dir?}: files written, and why not\n"
            "- analyze_traceability {model_path}: coverage and issues\n"
            "- safety_check {model_path, standard?}: output of `arclang safety`\n"
            "- extract_metrics {model_path}: element counts\n"
            "- export_filtered {model_path, filter_type?}: part of the JSON export "
            "(components_only = logical components, requirements_only, traces_only, all)\n"
            "- compare_models {model1_path, model2_path}: semantic diff (see arclang_diff "
            "for its limit)\n"
            "- analyze_diagrams {diagram_paths}: size and element counts of SVG files\n"
            f"- enhance_diagram {{diagram_path, enhancements}}: {', '.join(ENHANCEMENTS)}; "
            "writes <name>_enhanced.svg\n"
            "`filter.max_items` caps the lists."
        ),
        inputSchema=schema(
            {
                "operations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "action": {
                                "type": "string",
                                "enum": list(ACTIONS),
                                "description": "Operation to perform",
                            },
                            "params": {
                                "type": "object",
                                "description": "Parameters of the operation",
                            },
                            "filter": {
                                "type": "object",
                                "description": 'Optional, e.g. {"max_items": 10}',
                            },
                        },
                        "required": ["action", "params"],
                    },
                    "description": "Operations, executed in order",
                },
                "stop_on_error": {
                    "type": "boolean",
                    "description": "Stop at the first operation that fails",
                    "default": False,
                },
                "return_summary_only": {
                    "type": "boolean",
                    "description": "Return only the status of each operation",
                    "default": False,
                },
            },
            "operations",
        ),
    ),
]


class OperationFailed(Exception):
    """An operation's command failed; the message is the compiler's."""


def _require(run: Run) -> Run:
    if not run.ok:
        raise OperationFailed(run.output or f"exit {run.returncode}")
    return run


class BatchTools:
    """Several operations in one call."""

    def __init__(
        self, compiler: ArcLangCompiler, generation: GenerationTools, workspace_root: Path
    ):
        self.compiler = compiler
        self.generation = generation
        self.workspace_root = workspace_root
        self._actions: Dict[str, Callable[[Dict[str, Any], int], Awaitable[Any]]] = {
            "compile_and_validate": self._compile_and_validate,
            "gate": self._gate,
            "generate_all_diagrams": self._generate_all_diagrams,
            "analyze_traceability": self._analyze_traceability,
            "safety_check": self._safety_check,
            "extract_metrics": self._extract_metrics,
            "export_filtered": self._export_filtered,
            "compare_models": self._compare_models,
            "analyze_diagrams": self._analyze_diagrams,
            "enhance_diagram": self._enhance_diagram,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute the batch tool."""
        if tool_name != "arclang_execute_batch":
            raise ValueError(f"Unknown batch tool: {tool_name}")
        operations = arguments.get("operations", [])
        results: List[Dict[str, Any]] = []
        for number, operation in enumerate(operations, 1):
            action = operation.get("action", "")
            entry: Dict[str, Any] = {"operation": number, "action": action}
            try:
                handler = self._actions.get(action)
                if handler is None:
                    raise OperationFailed(f"unknown action '{action}'; known: {', '.join(ACTIONS)}")
                limit = max(0, int(operation.get("filter", {}).get("max_items", 10)))
                entry.update(
                    status="success", result=await handler(operation.get("params", {}), limit)
                )
            except Exception as failure:  # noqa: BLE001 - one bad operation must not lose the others
                if isinstance(failure, KeyError):
                    reason = f"missing parameter {failure}"
                else:
                    reason = str(failure) or type(failure).__name__
                entry.update(status="error", error=reason)
            results.append(entry)
            if entry["status"] == "error" and arguments.get("stop_on_error", False):
                break

        failed = sum(1 for entry in results if entry["status"] == "error")
        summary = {
            "total_operations": len(operations),
            "executed": len(results),
            "successful": len(results) - failed,
            "failed": failed,
        }
        if arguments.get("return_summary_only", False):
            summary["operations"] = [
                {"action": entry["action"], "status": entry["status"]} for entry in results
            ]
            return json.dumps(summary, indent=2)
        return json.dumps({"summary": summary, "results": results}, indent=2)

    def _path(self, params: Dict[str, Any], key: str = "model_path") -> Path:
        return resolve_path(self.workspace_root, params[key])

    async def _compile_and_validate(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        run = _require(await self.compiler.check(self._path(params)))
        compilation = parse_block(run.stdout, "Compilation warnings:")
        traceability = parse_block(run.stdout, "Traceability warnings:")
        advisories = parse_block(run.stdout, "methodology advisories:")
        return {
            "compiles": True,
            "gate_blocking_warnings": [
                line for line in compilation if line.startswith(("metamodel:", "constraint:"))
            ][:limit],
            "methodology_advisory_count": len(advisories),
            "methodology_advisories": advisories[:limit],
            "compilation_warning_count": len(compilation),
            "compilation_warnings": compilation[:limit],
            "traceability_warning_count": len(traceability),
            "traceability_warnings": traceability[:limit],
            "metrics": parse_metrics(run.stdout),
            "traceability_coverage_percent": parse_coverage(run.stdout),
        }

    async def _gate(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        run = await self.compiler.gate(self._path(params), params.get("standard", "iso26262"))
        if run.ok and "\nPRODUCTION GATE: PASS" in run.stdout:
            return {"verdict": "PASS", "blockers": []}
        if run.ok or "\nPRODUCTION GATE: FAIL" not in run.stdout:
            raise OperationFailed(run.output or f"exit {run.returncode}")
        blockers = parse_blockers(run.stdout)
        return {"verdict": "FAIL", "blocker_count": len(blockers), "blockers": blockers[:limit]}

    async def _generate_all_diagrams(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        output_dir = resolve_path(self.workspace_root, params.get("output_dir", "./diagrams"))
        results = await self.generation.generate_all(self._path(params), output_dir)
        written = {name: result for name, result in results.items() if "path" in result}
        if not written:
            errors = sorted({str(result.get("error", "")) for result in results.values()})
            raise OperationFailed("no diagram was generated: " + "; ".join(errors))
        return {
            "output_dir": str(output_dir),
            "generated": len(written),
            "requested": len(results),
            "total_bytes": sum(result["bytes"] for result in written.values()),
            "diagrams": results,
        }

    async def _analyze_traceability(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        run = _require(await self.compiler.trace(self._path(params)))
        issues = parse_block(run.stdout, "issues found:")
        return {
            "traceability_coverage_percent": parse_coverage(run.stdout),
            "issue_count": len(issues),
            "issues": issues[:limit],
        }

    async def _safety_check(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        standard = params.get("standard", "iso26262")
        run = _require(await self.compiler.safety(self._path(params), standard))
        counts = {
            label.strip().lower().replace(" ", "_"): int(value)
            for label, value in re.findall(r"^\s+([A-Za-z ]+):\s+(\d+)\s*$", run.stdout, flags=re.M)
        }
        return {
            "standard": standard,
            **counts,
            "note": "counts of declared safety content, not a compliance verdict; use the gate action",
        }

    async def _extract_metrics(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        run = _require(await self.compiler.info(self._path(params)))
        return {
            **parse_metrics(run.stdout),
            "traceability_coverage_percent": parse_coverage(run.stdout),
        }

    async def _export_filtered(self, params: Dict[str, Any], limit: int) -> Any:
        model = self._path(params)
        with tempfile.TemporaryDirectory(prefix="arclang-batch-") as directory:
            exported = Path(directory) / "model.json"
            _require(await self.compiler.export(model, "json", exported))
            data = json.loads(exported.read_text())
        filter_type = params.get("filter_type", "components_only")
        if filter_type == "components_only":
            components = [
                component
                for layer in data.get("logical_architecture", [])
                for component in layer.get("components", [])
            ]
            return {
                "layer": "logical architecture",
                "logical_component_count": len(components),
                "components": components[:limit],
            }
        if filter_type == "requirements_only":
            requirements = [
                requirement
                for layer in data.get("system_analysis", [])
                for requirement in layer.get("requirements", [])
            ]
            return {"requirement_count": len(requirements), "requirements": requirements[:limit]}
        if filter_type == "traces_only":
            traces = data.get("traces", [])
            return {"trace_count": len(traces), "traces": traces[:limit]}
        if filter_type == "all":
            return data
        raise OperationFailed(
            f"unknown filter_type '{filter_type}'; known: components_only, requirements_only, "
            "traces_only, all"
        )

    async def _compare_models(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        run = await self.compiler.diff(
            self._path(params, "model1_path"), self._path(params, "model2_path")
        )
        changes = parse_diff(run)
        if changes is None:
            raise OperationFailed(run.output or f"exit {run.returncode}")
        return {
            "counts": {key: len(value) for key, value in changes.items()},
            "changes": {key: value[:limit] for key, value in changes.items()},
        }

    async def _analyze_diagrams(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        diagrams: List[Dict[str, Any]] = []
        for given in params.get("diagram_paths", []):
            path = resolve_path(self.workspace_root, given)
            if not path.is_file():
                diagrams.append({"path": given, "error": "file not found"})
                continue
            content = path.read_text()
            root = re.search(r"<svg\b[^>]*>", content)
            width = re.search(r'\bwidth="(\d+)"', root.group(0)) if root else None
            height = re.search(r'\bheight="(\d+)"', root.group(0)) if root else None
            elements = {
                tag: content.count(f"<{tag} ")
                for tag in ("g", "rect", "circle", "line", "path", "text")
            }
            diagrams.append(
                {
                    "path": str(path),
                    "bytes": len(content.encode()),
                    "dimensions": (
                        f"{width.group(1)}x{height.group(1)}" if width and height else None
                    ),
                    "elements": {**elements, "total": sum(elements.values())},
                    "fill_colours": sorted(set(re.findall(r'fill="#([0-9A-Fa-f]{6})"', content)))[
                        :limit
                    ],
                }
            )
        return {"diagrams": diagrams}

    async def _enhance_diagram(self, params: Dict[str, Any], limit: int) -> Dict[str, Any]:
        source = self._path(params, "diagram_path")
        if not source.is_file():
            raise OperationFailed(f"diagram not found: {source}")
        content = source.read_text()
        modified = content
        applied: List[str] = []
        unknown: List[str] = []
        without_effect: List[str] = []
        for enhancement in params.get("enhancements", []):
            if enhancement not in ENHANCEMENTS:
                unknown.append(enhancement)
                continue
            enhanced = self._apply(enhancement, modified)
            (applied if enhanced != modified else without_effect).append(enhancement)
            modified = enhanced
        target = source.with_name(f"{source.stem}_enhanced.svg")
        if applied:
            target.write_text(modified)
        return {
            "original": str(source),
            "enhanced": str(target) if applied else None,
            "enhancements_applied": applied,
            "enhancements_without_effect": without_effect,
            "enhancements_not_implemented": unknown,
            "bytes_before": len(content.encode()),
            "bytes_after": len(modified.encode()),
        }

    @staticmethod
    def _apply(enhancement: str, svg: str) -> str:
        if enhancement == "enlarge_text":
            return re.sub(
                r'font-size="(\d+)"', lambda m: f'font-size="{int(int(m.group(1)) * SCALE)}"', svg
            )
        if enhancement == "improve_contrast":

            def darken(match: "re.Match[str]") -> str:
                red, green, blue = (int(match.group(1)[i : i + 2], 16) for i in (0, 2, 4))
                if red + green + blue <= LIGHT_COLOUR_SUM:
                    return match.group(0)
                return 'fill="#{:02x}{:02x}{:02x}"'.format(
                    int(red * DARKEN), int(green * DARKEN), int(blue * DARKEN)
                )

            return re.sub(r'fill="#([0-9A-Fa-f]{6})"', darken, svg)
        # add_grid
        if "<defs>" in svg:
            with_grid = svg.replace("<defs>", f"<defs>\n{GRID_PATTERN}", 1)
        else:
            with_grid = svg.replace("</svg>", f"<defs>\n{GRID_PATTERN}\n</defs>\n</svg>", 1)
        return re.sub(
            r'<rect x="0" y="0"([^>]+)fill="#FFFFFF"',
            r'<rect x="0" y="0"\1fill="url(#grid)"',
            with_grid,
        )
