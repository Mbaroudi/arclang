"""
Core tools for ArcLang compilation, validation, and analysis.
"""

import json
from pathlib import Path
from typing import Any, Dict

from ..compiler.wrapper import ArcLangCompiler


class CoreTools:
    """Core ArcLang tools (compile, validate, trace, export)."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a core tool."""
        if tool_name == "arclang_compile":
            return await self._compile(arguments)
        elif tool_name == "arclang_validate":
            return await self._validate(arguments)
        elif tool_name == "arclang_trace_analysis":
            return await self._trace_analysis(arguments)
        elif tool_name == "arclang_export_diagram":
            return await self._export_diagram(arguments)
        elif tool_name == "arclang_info":
            return await self._info(arguments)
        else:
            raise ValueError(f"Unknown core tool: {tool_name}")

    async def _compile(self, args: Dict[str, Any]) -> str:
        """Compile ArcLang model."""
        model_path = self._resolve_path(args["model_path"])
        validate = args.get("validate", True)
        optimize = args.get("optimize", False)

        result = await self.compiler.compile(
            model_path,
            validate=validate,
            optimize=optimize
        )

        if result["success"]:
            output = f"✅ Compilation successful\n\n"
            output += f"**Model**: {model_path.name}\n"
            
            if "metrics" in result:
                metrics = result["metrics"]
                output += f"**Requirements**: {metrics.get('requirements', 0)}\n"
                output += f"**Components**: {metrics.get('components', 0)}\n"
                output += f"**Functions**: {metrics.get('functions', 0)}\n"
                output += f"**Traces**: {metrics.get('traces', 0)}\n"
            
            if validate and "validation" in result:
                validation = result["validation"]
                output += f"\n**Validation**: {validation.get('status', 'OK')}\n"
                if validation.get('warnings'):
                    output += f"⚠️  Warnings: {len(validation['warnings'])}\n"
            
            output += f"\n**Output**: {result.get('output_path', 'N/A')}"
            return output
        else:
            return f"❌ Compilation failed\n\n**Errors**:\n{result.get('errors', 'Unknown error')}"

    async def _validate(self, args: Dict[str, Any]) -> str:
        """Validate ArcLang model."""
        model_path = self._resolve_path(args["model_path"])
        strict = args.get("strict", False)

        result = await self.compiler.validate(model_path, strict=strict)

        if result["valid"]:
            output = f"✅ Validation passed\n\n"
            output += f"**Model**: {model_path.name}\n"
            output += f"**Syntax**: Valid\n"
            output += f"**Semantics**: Valid\n"
            
            if result.get("warnings"):
                output += f"\n⚠️  **Warnings** ({len(result['warnings'])}):\n"
                for warning in result["warnings"][:5]:  # Show first 5
                    output += f"  - {warning}\n"
            
            return output
        else:
            output = f"❌ Validation failed\n\n"
            output += f"**Model**: {model_path.name}\n\n"
            output += f"**Errors** ({len(result.get('errors', []))}):\n"
            for error in result.get("errors", [])[:10]:  # Show first 10
                output += f"  - {error}\n"
            return output

    async def _trace_analysis(self, args: Dict[str, Any]) -> str:
        """Analyze traceability."""
        model_path = self._resolve_path(args["model_path"])
        show_gaps = args.get("show_gaps", True)
        matrix = args.get("matrix", False)

        result = await self.compiler.trace_analysis(
            model_path,
            show_gaps=show_gaps,
            matrix=matrix
        )

        output = f"🔗 **Traceability Analysis**\n\n"
        output += f"**Model**: {model_path.name}\n"
        output += f"**Coverage**: {result.get('coverage', 0)}%\n"
        output += f"**Total Traces**: {result.get('total_traces', 0)}\n\n"

        if show_gaps and result.get("gaps"):
            gaps = result["gaps"]
            output += f"⚠️  **Gaps Found** ({len(gaps)}):\n\n"
            
            untraced_req = gaps.get("untraced_requirements", [])
            if untraced_req:
                output += f"**Untraced Requirements** ({len(untraced_req)}):\n"
                for req in untraced_req[:5]:
                    output += f"  - {req['id']}: {req.get('description', 'N/A')}\n"
                output += "\n"
            
            untraced_comp = gaps.get("untraced_components", [])
            if untraced_comp:
                output += f"**Untraced Components** ({len(untraced_comp)}):\n"
                for comp in untraced_comp[:5]:
                    output += f"  - {comp['id']}: {comp.get('name', 'N/A')}\n"
                output += "\n"
        
        if result.get("coverage", 0) >= 90:
            output += "✅ Good traceability coverage"
        elif result.get("coverage", 0) >= 70:
            output += "⚠️  Moderate traceability coverage - consider adding more traces"
        else:
            output += "❌ Low traceability coverage - significant gaps detected"

        return output

    async def _export_diagram(self, args: Dict[str, Any]) -> str:
        """Export architecture diagram."""
        model_path = self._resolve_path(args["model_path"])
        format_type = args.get("format", "html")
        output_path = args.get("output_path")

        result = await self.compiler.export_diagram(
            model_path,
            format_type=format_type,
            output_path=output_path
        )

        if result["success"]:
            output = f"✅ Diagram generated successfully\n\n"
            output += f"**Model**: {model_path.name}\n"
            output += f"**Format**: {format_type}\n"
            output += f"**Output**: {result['output_path']}\n"
            
            if result.get("metrics"):
                metrics = result["metrics"]
                output += f"\n**Components**: {metrics.get('components', 0)}\n"
                output += f"**Connections**: {metrics.get('connections', 0)}\n"
            
            return output
        else:
            return f"❌ Diagram generation failed\n\n{result.get('error', 'Unknown error')}"

    async def _info(self, args: Dict[str, Any]) -> str:
        """Get model information and metrics."""
        model_path = self._resolve_path(args["model_path"])
        detailed = args.get("detailed", False)

        result = await self.compiler.info(model_path, detailed=detailed)

        output = f"📊 **Model Information**\n\n"
        output += f"**Path**: {model_path}\n"
        output += f"**Size**: {result.get('size_kb', 0)} KB\n\n"

        metrics = result.get("metrics", {})
        output += f"**Metrics**:\n"
        output += f"  - Requirements: {metrics.get('requirements', 0)}\n"
        output += f"  - Components: {metrics.get('components', 0)}\n"
        output += f"  - Functions: {metrics.get('functions', 0)}\n"
        output += f"  - Traces: {metrics.get('traces', 0)}\n"
        output += f"  - Actors: {metrics.get('actors', 0)}\n"
        output += f"  - Nodes: {metrics.get('nodes', 0)}\n\n"

        if metrics.get("safety"):
            safety = metrics["safety"]
            output += f"**Safety Metrics**:\n"
            output += f"  - ASIL-D: {safety.get('asil_d', 0)}\n"
            output += f"  - ASIL-C: {safety.get('asil_c', 0)}\n"
            output += f"  - ASIL-B: {safety.get('asil_b', 0)}\n"
            output += f"  - ASIL-A: {safety.get('asil_a', 0)}\n\n"

        if result.get("coverage"):
            coverage = result["coverage"]
            output += f"**Traceability Coverage**: {coverage.get('percentage', 0)}%\n"

        return output

    def _resolve_path(self, path_str: str) -> Path:
        """Resolve path relative to workspace root."""
        path = Path(path_str)
        if not path.is_absolute():
            path = self.workspace_root / path
        return path
