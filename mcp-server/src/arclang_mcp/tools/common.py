"""
What every tool group shares: path resolution and the shape of a report.
"""

import difflib
import json
from pathlib import Path
from typing import Any, Dict, List, Optional

from ..compiler.wrapper import Run

MODEL_PATH = {"type": "string", "description": "Path to the .arc model file"}


def resolve_path(workspace_root: Path, path: str) -> Path:
    """An absolute path as given; a relative one from the workspace root; `~` expanded."""
    if not path.strip():
        # "" would be the workspace itself: `format --write` on it rewrites every file.
        raise InvalidArguments("a path must not be empty")
    expanded = Path(path).expanduser()
    return expanded if expanded.is_absolute() else workspace_root.resolve() / expanded


def schema(properties: Dict[str, Any], *required: str) -> Dict[str, Any]:
    """An object JSON schema for a tool's arguments."""
    return {"type": "object", "properties": properties, "required": list(required)}


def report(title: str, run: Run, verdict: Optional[str] = None) -> str:
    """A tool answer: what was asked, how it ended, and what the compiler printed.

    The compiler's own words are returned as they are; the only thing added
    is the outcome line, read from the exit code.
    """
    outcome = verdict or ("succeeded" if run.ok else f"FAILED (exit {run.returncode})")
    body = run.output or "(the compiler printed nothing)"
    return f"{title}: {outcome}\n\n{body}"


def preview(title: str, source: Path, run: Run) -> str:
    """A change the compiler printed instead of writing, as a diff against the file.

    The compiler's edits (`set`, `unset`, `rename`, `format`, `transition`)
    print the edited model on stdout unless told to write: the answer is a
    unified diff of the file against that text, and says it was NOT written.
    """
    if not run.ok:
        return report(title, run)
    try:
        before = source.read_text()
    except OSError as failure:
        return f"{title}: cannot read {source}: {failure}"
    if before == run.stdout:
        return f"{title}: no change"
    diff = difflib.unified_diff(
        before.splitlines(keepends=True),
        run.stdout.splitlines(keepends=True),
        fromfile=source.name,
        tofile=f"{source.name} (not written)",
    )
    return f"{title}: accepted, NOT written (pass write: true to apply)\n\n{''.join(diff)}"


def input_schema(tool: Any) -> Dict[str, Any]:
    """A tool's argument schema, under either generation of the MCP SDK."""
    return getattr(tool, "inputSchema", None) or tool.input_schema


def mime_type(resource: Any) -> Optional[str]:
    """A resource's media type, under either generation of the MCP SDK."""
    return getattr(resource, "mimeType", None) or getattr(resource, "mime_type", None)


class InvalidArguments(ValueError):
    """The arguments of a call do not match the tool's schema."""


JSON_TYPES = {
    "string": str,
    "boolean": bool,
    "integer": int,
    "number": (int, float),
    "array": list,
    "object": dict,
}


def check_arguments(tool: Any, arguments: Dict[str, Any]) -> None:
    """Refuse arguments that are not what the tool's schema declares.

    The server checks this itself because not every MCP SDK does, and the
    handlers must be able to trust a flag: `write: "false"` is a string, and
    a string must never be read as a yes to writing or sending. Nested
    objects and arrays are checked the same way, so a batch operation that
    is not an object is refused before it can abort the whole batch.
    """
    problems = check_value(input_schema(tool), arguments, "")
    if problems:
        raise InvalidArguments(f"invalid arguments for {tool.name}: " + "; ".join(problems))


def check_value(expected: Dict[str, Any], value: Any, where: str) -> List[str]:
    """The ways `value` departs from the schema `expected`; `where` names it."""
    name = where or "the arguments"
    declared_type = expected.get("type", "")
    python_type = JSON_TYPES.get(declared_type)
    is_bool = isinstance(value, bool)
    if python_type is not None and (
        not isinstance(value, python_type) or (is_bool and declared_type != "boolean")
    ):
        return [f"{name} must be {ARTICLE.get(declared_type, 'a')} {declared_type}, got {json.dumps(value)}"]
    if isinstance(value, float) and value != value:
        return [f"{name} must be a number, got NaN"]
    if "enum" in expected and value not in expected["enum"]:
        return [f"{name} must be one of {', '.join(map(str, expected['enum']))}, got {json.dumps(value)}"]
    if declared_type == "array":
        items = expected.get("items")
        return (
            [p for n, item in enumerate(value) for p in check_value(items, item, f"{name}[{n}]")]
            if items
            else []
        )
    if declared_type != "object":
        return []
    properties = expected.get("properties", {})
    problems = []
    for key in expected.get("required", []):
        if key not in value:
            problems.append(f"'{key}' is required" + (f" in {where}" if where else ""))
        elif value[key] == "":
            problems.append(f"'{key}' must not be empty" + (f" in {where}" if where else ""))
    for key, item in value.items():
        child = properties.get(key)
        if child is None:
            if properties:
                problems.append(f"'{key}' is not an argument of " + (where or "this tool"))
            continue
        problems += check_value(child, item, f"{where}.{key}" if where else f"'{key}'")
    return problems


ARTICLE = {"integer": "an", "object": "an", "array": "an"}
