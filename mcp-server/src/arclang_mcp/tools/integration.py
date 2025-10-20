"""
Integration tools for Git, PLM, and requirements management systems.
"""

from pathlib import Path
from typing import Any, Dict


class IntegrationTools:
    """Integration tools for Git, PLM, DOORS, etc."""

    def __init__(self, compiler: Any, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute an integration tool."""
        if tool_name == "arclang_git_merge":
            return await self._git_merge(arguments)
        elif tool_name == "arclang_plm_sync":
            return await self._plm_sync(arguments)
        else:
            raise ValueError(f"Unknown integration tool: {tool_name}")

    async def _git_merge(self, args: Dict[str, Any]) -> str:
        """Semantic merge assistance."""
        base_path = Path(args["base_path"])
        ours_path = Path(args["ours_path"])
        theirs_path = Path(args["theirs_path"])

        result = await self.compiler.semantic_merge(
            base_path=base_path,
            ours_path=ours_path,
            theirs_path=theirs_path
        )

        output = f"🔀 **Semantic Merge Analysis**\n\n"
        
        if result.get("clean_merge"):
            output += "✅ **Clean merge** - No conflicts!\n\n"
            output += f"**Elements merged**: {result.get('merged_count', 0)}\n"
        else:
            conflicts = result.get("conflicts", [])
            output += f"⚠️  **Conflicts detected**: {len(conflicts)}\n\n"
            
            for conflict in conflicts[:5]:
                output += f"**{conflict['id']}**:\n"
                output += f"  - Type: {conflict['type']}\n"
                output += f"  - Ours: {conflict.get('ours_summary', 'N/A')}\n"
                output += f"  - Theirs: {conflict.get('theirs_summary', 'N/A')}\n"
                output += f"  - Suggestion: {conflict.get('suggestion', 'Manual resolution needed')}\n\n"

        return output

    async def _plm_sync(self, args: Dict[str, Any]) -> str:
        """PLM synchronization."""
        model_path = Path(args["model_path"])
        system = args["system"]
        operation = args.get("operation", "pull")

        result = await self.compiler.plm_sync(
            model_path=model_path,
            system=system,
            operation=operation
        )

        output = f"🔄 **PLM Synchronization**\n\n"
        output += f"**System**: {system.upper()}\n"
        output += f"**Operation**: {operation}\n"
        output += f"**Model**: {model_path.name}\n\n"

        if result.get("success"):
            output += "✅ Synchronization successful\n\n"
            if operation == "pull":
                output += f"**Changes pulled**: {result.get('changes_count', 0)}\n"
            elif operation == "push":
                output += f"**Changes pushed**: {result.get('changes_count', 0)}\n"
        else:
            output += f"❌ Synchronization failed\n\n{result.get('error', 'Unknown error')}"

        return output
