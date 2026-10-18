"""
Edit tools: format a model and change it one element at a time.

They are the compiler's own lossless edits (`arclang format`, `set`, `unset`,
`rename`): comments, order and layout are kept, and an edit is refused
unless the edited model compiles and holds exactly what was asked. Nothing
is written unless the caller passes `write: true`; otherwise the tool
returns the text the file would have.
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import ArcLangCompiler, Run
from .common import MODEL_PATH, preview, report, resolve_path, schema

WRITE = {
    "type": "boolean",
    "description": "Rewrite the file in place. Default: change nothing and return the diff",
    "default": False,
}
ELEMENT = {
    "type": "string",
    "description": "Identifier of the element, or its name when it writes no id",
}

TOOLS: List[Tool] = [
    Tool(
        name="arclang_format",
        description=(
            "Format a .arc file, or every .arc file of a directory (`arclang format`). "
            "Whitespace only: comments and order are kept, and the result is re-read to "
            "prove that nothing else changed."
        ),
        inputSchema=schema(
            {
                "path": {"type": "string", "description": "A .arc file or a directory"},
                "write": WRITE,
            },
            "path",
        ),
    ),
    Tool(
        name="arclang_set_attribute",
        description=(
            "Set one attribute of one element (`arclang set`). The value is ArcLang "
            "source: '\"text\"', '25 ms', '[A, B]'. Refused when the element or the "
            "attribute is unknown or ambiguous, when the edited model does not compile, or "
            "when the compiled element does not show the value."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": ELEMENT,
                "key": {"type": "string", "description": "Attribute name"},
                "value": {"type": "string", "description": "Value, as ArcLang source"},
                "write": WRITE,
            },
            "model_path",
            "element",
            "key",
            "value",
        ),
    ),
    Tool(
        name="arclang_unset_attribute",
        description="Remove one attribute of one element (`arclang unset`). Same guards as "
        "arclang_set_attribute.",
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": ELEMENT,
                "key": {"type": "string", "description": "Attribute name"},
                "write": WRITE,
            },
            "model_path",
            "element",
            "key",
        ),
    ),
    Tool(
        name="arclang_rename",
        description=(
            "Rename one element (`arclang rename`). Refused when the element is known by "
            "its name elsewhere, because its identity must not change: give it an `id` first."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": {"type": "string", "description": "Identifier of the element"},
                "name": {"type": "string", "description": "The new name"},
                "write": WRITE,
            },
            "model_path",
            "element",
            "name",
        ),
    ),
]


class EditTools:
    """Format and scripted edits of model files."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_format": self._format,
            "arclang_set_attribute": self._set,
            "arclang_unset_attribute": self._unset,
            "arclang_rename": self._rename,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute an edit tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown edit tool: {tool_name}")
        return await handler(arguments)

    async def _format(self, args: Dict[str, Any]) -> str:
        target = resolve_path(self.workspace_root, args["path"])
        write = args.get("write", False)
        if write:
            return report(
                f"Formatting of {target.name}", await self.compiler.format(target, write=True)
            )
        if target.is_dir():
            run = await self.compiler.format(target, check=True)
            verdict = (
                "all formatted" if run.ok else "some files are not formatted (nothing written)"
            )
            return report(f"Format check of {target.name}", run, verdict)
        return preview(f"Formatting of {target.name}", target, await self.compiler.format(target))

    async def _set(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        write = args.get("write", False)
        run = await self.compiler.set_attribute(
            model, args["element"], args["key"], args["value"], write=write
        )
        return self._outcome(f"Set {args['element']}.{args['key']}", model, run, write)

    async def _unset(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        write = args.get("write", False)
        run = await self.compiler.unset_attribute(model, args["element"], args["key"], write=write)
        return self._outcome(f"Unset {args['element']}.{args['key']}", model, run, write)

    async def _rename(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        write = args.get("write", False)
        run = await self.compiler.rename(model, args["element"], args["name"], write=write)
        return self._outcome(f"Rename {args['element']} to {args['name']}", model, run, write)

    def _outcome(self, title: str, model: Path, run: Run, write: bool) -> str:
        if write:
            verdict = None
            if run.ok:
                unchanged = run.stdout.strip().endswith("unchanged")
                verdict = "no change, nothing written" if unchanged else f"written to {model.name}"
            return report(title, run, verdict)
        return preview(title, model, run)
