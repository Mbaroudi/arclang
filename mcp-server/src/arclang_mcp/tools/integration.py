"""
Integration tools.

The compiler has no merge and no PLM connector (`arclang sync` is a stub),
so the tools below say so instead of pretending. What exists for working
with versions is the semantic diff (`arclang_diff`) and the API writes of
`arclang serve --allow-write`, one git commit per write.
"""

from typing import Any, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import ArcLangCompiler
from .common import schema

MERGE_UNAVAILABLE = (
    "Semantic merge is not available: the arclang CLI has no merge subcommand. "
    "Merge the .arc files with git, then run arclang_diff between each parent and "
    "the result to read the outcome by stable identity, and arclang_validate on it."
)
PLM_UNAVAILABLE = (
    "PLM synchronization is not available: `arclang sync` is not implemented in the compiler."
)

TOOLS: List[Tool] = [
    Tool(
        name="arclang_git_merge",
        description=(
            "NOT AVAILABLE: the arclang CLI has no merge subcommand, so the tool answers so "
            "and changes nothing. Use arclang_diff to compare versions by stable identity."
        ),
        inputSchema=schema(
            {
                "base_path": {"type": "string", "description": "Path to base version"},
                "ours_path": {"type": "string", "description": "Path to our version"},
                "theirs_path": {"type": "string", "description": "Path to their version"},
            },
            "base_path",
            "ours_path",
            "theirs_path",
        ),
    ),
]


class IntegrationTools:
    """Tools the compiler does not back yet: they answer honestly."""

    def __init__(self, compiler: ArcLangCompiler, config: Dict[str, Any]):
        self.compiler = compiler
        self.config = config

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute an integration tool."""
        if tool_name == "arclang_git_merge":
            return MERGE_UNAVAILABLE
        if tool_name == "arclang_plm_sync":
            return PLM_UNAVAILABLE
        raise ValueError(f"Unknown integration tool: {tool_name}")
