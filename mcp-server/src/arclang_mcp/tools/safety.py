"""
Safety validation tools for ISO 26262, DO-178C, IEC 61508.
"""

from pathlib import Path
from typing import Any, Dict


class SafetyTools:
    """Safety compliance and analysis tools."""

    def __init__(self, compiler: Any, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a safety tool."""
        if tool_name == "arclang_safety_check":
            return await self._safety_check(arguments)
        elif tool_name == "arclang_hazard_analysis":
            return await self._hazard_analysis(arguments)
        else:
            raise ValueError(f"Unknown safety tool: {tool_name}")

    async def _safety_check(self, args: Dict[str, Any]) -> str:
        """Validate safety compliance."""
        model_path = Path(args["model_path"])
        standard = args["standard"]
        generate_report = args.get("generate_report", False)

        result = await self.compiler.safety_check(
            model_path,
            standard=standard,
            generate_report=generate_report
        )

        output = f"🛡️  **Safety Compliance Check**\n\n"
        output += f"**Standard**: {standard.upper()}\n"
        output += f"**Model**: {model_path.name}\n\n"

        if result.get("compliant"):
            output += f"✅ **Status**: Compliant\n\n"
        else:
            output += f"❌ **Status**: Non-compliant\n\n"

        issues = result.get("issues", [])
        if issues:
            output += f"**Issues Found** ({len(issues)}):\n\n"
            for issue in issues[:10]:
                severity = issue.get("severity", "warning")
                icon = "🔴" if severity == "error" else "⚠️"
                output += f"{icon} **{issue['title']}**\n"
                output += f"   {issue['description']}\n\n"

        if result.get("recommendations"):
            output += "\n**Recommendations**:\n"
            for rec in result["recommendations"][:5]:
                output += f"  - {rec}\n"

        if generate_report and result.get("report_path"):
            output += f"\n📄 **Detailed Report**: {result['report_path']}"

        return output

    async def _hazard_analysis(self, args: Dict[str, Any]) -> str:
        """Perform hazard analysis."""
        model_path = Path(args["model_path"])
        standard = args.get("standard", "iso26262")

        result = await self.compiler.hazard_analysis(
            model_path,
            standard=standard
        )

        output = f"⚠️  **Hazard Analysis (HARA)**\n\n"
        output += f"**Standard**: {standard.upper()}\n"
        output += f"**Model**: {model_path.name}\n\n"

        hazards = result.get("hazards", [])
        output += f"**Hazards Identified**: {len(hazards)}\n\n"

        for hazard in hazards[:5]:
            output += f"**{hazard['id']}**: {hazard['description']}\n"
            output += f"  - Severity: {hazard['severity']}\n"
            output += f"  - Exposure: {hazard.get('exposure', 'N/A')}\n"
            output += f"  - Controllability: {hazard.get('controllability', 'N/A')}\n"
            output += f"  - ASIL: {hazard.get('asil', 'N/A')}\n\n"

        return output
