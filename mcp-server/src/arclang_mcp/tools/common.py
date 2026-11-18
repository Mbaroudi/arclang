"""
What every tool group shares: path resolution and the shape of a report.
"""

import difflib
import json
import os
import re
from pathlib import Path
from typing import Any, Dict, List, Optional

from ..compiler.wrapper import Run

MODEL_PATH = {"type": "string", "description": "Path to the .arc model file"}


# The other directories the server works in, separated like PATH; `*` is any path.
ROOTS = "ARCLANG_MCP_ROOTS"
ANYWHERE = "*"


def allowed_roots(workspace_root: Path) -> Optional[List[Path]]:
    """The workspace and the directories named in ARCLANG_MCP_ROOTS; None when any path is."""
    named = [entry.strip() for entry in os.environ.get(ROOTS, "").split(os.pathsep)]
    if ANYWHERE in named:
        return None
    return [workspace_root.resolve()] + [
        Path(entry).expanduser().resolve() for entry in named if entry
    ]


def resolve_path(workspace_root: Path, path: str) -> Path:
    """A relative path from the workspace root, `~` expanded; refused outside the roots.

    The agent gives the paths, and what it read (a model, a document, a page) can ask it
    for any of them: the server reads and writes under the workspace and under the
    directories it was told, and nowhere else. The place is judged where the path leads,
    links and `..` followed; the path is returned as it was written.
    """
    if not path.strip():
        # "" would be the workspace itself: `format --write` on it rewrites every file.
        raise InvalidArguments("a path must not be empty")
    expanded = Path(path).expanduser()
    target = expanded if expanded.is_absolute() else workspace_root.resolve() / expanded
    roots = allowed_roots(workspace_root)
    if roots is not None:
        leads_to = target.resolve()
        if not any(leads_to == root or root in leads_to.parents for root in roots):
            raise InvalidArguments(
                f"'{path}' is outside the workspace ({roots[0]}): the server reads and writes "
                f"under the workspace and under the directories named in {ROOTS} "
                f"(separated by '{os.pathsep}'; '{ANYWHERE}' is any path)"
            )
    return target


class CommandLine:
    """The arguments of one `arclang <subcommand>`, built from a tool's arguments.

    Every option is written `--option=value`, so that a value such as `--help` or
    `-5` is the value of its option and never an option of its own; the positional
    arguments follow `--`. Every path, read or written, goes through `resolve_path`.
    """

    def __init__(self, workspace_root: Path, args: Dict[str, Any], *command: str):
        self.workspace_root = workspace_root
        self.args = args
        self.parts: List[str] = list(command)

    def given(self, key: str) -> bool:
        return self.args.get(key) is not None and self.args.get(key) != ""

    def flag(self, key: str, option: str) -> "CommandLine":
        """`option` when the boolean argument `key` is true."""
        if self.args.get(key) is True:
            self.parts.append(option)
        return self

    def add(self, *options: str) -> "CommandLine":
        """Options the tool always passes."""
        self.parts += options
        return self

    def value(self, key: str, option: str) -> "CommandLine":
        """`option=<value>` when the argument `key` is given."""
        if self.given(key):
            value = self.args[key]
            text = f"{value:g}" if isinstance(value, float) else str(value)
            self.parts.append(f"{option}={text}")
        return self

    def each(self, key: str, option: str) -> "CommandLine":
        """`option=<item>` for each item of the list argument `key`."""
        for item in self.args.get(key) or []:
            self.parts.append(f"{option}={item}")
        return self

    def joined(self, key: str, option: str) -> "CommandLine":
        """`option=<a,b,c>` for a list the CLI reads comma separated."""
        items = [str(item) for item in self.args.get(key) or []]
        if any("," in item for item in items):
            raise InvalidArguments(f"an item of '{key}' must not hold a comma")
        if items:
            self.parts.append(f"{option}={','.join(items)}")
        return self

    def path(self, key: str, option: str, make_parent: bool = False) -> Optional[Path]:
        """`option=<path>` when `key` is given: the path resolved, refused outside the roots."""
        if not self.given(key):
            return None
        target = resolve_path(self.workspace_root, self.args[key])
        if make_parent:
            target.parent.mkdir(parents=True, exist_ok=True)
        self.parts.append(f"{option}={target}")
        return target

    def end(self, *positional: Any) -> List[str]:
        """The finished argument list, the positional arguments after `--`."""
        return [*self.parts, "--", *(str(item) for item in positional)]


def bounded_seconds(args: Dict[str, Any], key: str, default: int, most: int) -> int:
    """A number of seconds the call gives, from 1 to `most`; `default` when it gives none."""
    seconds = args.get(key, default)
    if isinstance(seconds, bool) or not isinstance(seconds, int) or not 1 <= seconds <= most:
        raise InvalidArguments(f"'{key}' is a number of seconds from 1 to {most}")
    return seconds


# Set to 1 by the person who starts the server to let a call name a program to run.
ALLOW_COMMANDS = "ARCLANG_MCP_ALLOW_COMMANDS"
SERIAL_PORT = re.compile(r"^(/dev/[A-Za-z0-9._-]+|COM[0-9]+)$")
PROGRAM_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._+-]*$")


def commands_allowed() -> bool:
    return os.environ.get(ALLOW_COMMANDS, "").strip().lower() in ("1", "true", "yes")


def command_argument(args: Dict[str, Any], key: str, instead: str) -> Optional[str]:
    """A command line the call asks the compiler to run (a proposer, a stand-in for a board).

    What an agent read can ask it for any command, as for any path: the server
    passes one on only when the person who started it said so. `instead` says
    what works without that.
    """
    command = args.get(key)
    if command is None or command == "":
        return None
    if not commands_allowed():
        raise InvalidArguments(
            f"'{key}' names a program to run, and this server runs none a call names unless it "
            f"was started with {ALLOW_COMMANDS}=1. {instead}"
        )
    return str(command)


def program_name(value: Any, key: str) -> str:
    """A program found on the PATH by its bare name (a C compiler: gcc, clang, gcc-15)."""
    if not isinstance(value, str) or not PROGRAM_NAME.match(value):
        raise InvalidArguments(
            f"'{key}' is the bare name of a program on the PATH (gcc, clang, gcc-15), not a path "
            f"or a command line, got {json.dumps(value)}"
        )
    return value


def serial_port(args: Dict[str, Any], key: str) -> Optional[str]:
    """A serial port, by its device name: the one path a tool takes that is not a file."""
    port = args.get(key)
    if port is None or port == "":
        return None
    if not isinstance(port, str) or not SERIAL_PORT.match(port):
        raise InvalidArguments(
            f"'{key}' is a serial port (/dev/cu.usbmodem1101, /dev/ttyACM0, COM3), "
            f"got {json.dumps(port)}"
        )
    return port


def repository_root(workspace_root: Path, directory: Path) -> Path:
    """The top of the git repository `directory` is in (itself when in none), refused outside the roots.

    The commands that write for a repository (`ci init`, `baseline create`) write at
    its top, wherever in it they are run: the top is a path the server writes to.
    """
    start = directory.resolve()
    top = next((place for place in (start, *start.parents) if (place / ".git").exists()), start)
    resolve_path(workspace_root, str(top))
    return top


def listing(directory: Path, limit: int = 60) -> str:
    """The files under a directory the compiler filled, with their sizes."""
    files = sorted(path for path in directory.rglob("*") if path.is_file())
    lines = [f"- {path.relative_to(directory)} ({path.stat().st_size} bytes)" for path in files[:limit]]
    if len(files) > limit:
        lines.append(f"- ... and {len(files) - limit} more")
    return "\n".join(lines) or "(no file)"


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
    return diff_against(title, source.name, before, run.stdout)


def diff_against(title: str, name: str, before: str, after: str) -> str:
    """The answer for an accepted edit that was not written: `after` as a diff of `before`."""
    if before == after:
        return f"{title}: no change"
    diff = difflib.unified_diff(
        before.splitlines(keepends=True),
        after.splitlines(keepends=True),
        fromfile=name,
        tofile=f"{name} (not written)",
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
        extra = expected.get("additionalProperties")
        if child is None and isinstance(extra, dict):
            # A map (machine -> step time): every value checked against the one schema.
            problems += check_value(extra, item, f"{where}.{key}" if where else f"'{key}'")
            continue
        if child is None:
            if properties:
                problems.append(f"'{key}' is not an argument of " + (where or "this tool"))
            continue
        problems += check_value(child, item, f"{where}.{key}" if where else f"'{key}'")
    return problems


ARTICLE = {"integer": "an", "object": "an", "array": "an"}
