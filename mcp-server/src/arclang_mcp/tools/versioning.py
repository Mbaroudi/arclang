"""
Versions of a model: the merge by element, the history of one element, a
model (or some of its elements) as it was at a git revision, and baselines.

Each tool is a subcommand of the compiler (`arclang merge`, `history`,
`restore`, `baseline`) and returns what it printed. They read git and never
rewrite its history: a merge and a restore print a text unless the call
names where to write it; only `arclang_baseline` with `action: create`
changes the repository (a `baselines/<name>/` directory and a tag), and it
does so only when asked by that name.

The semantic diff between two files or two revisions is `arclang_diff`, and
the model as committed at a revision is `arclang_workbench` with `rev`.
"""

import json
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import DIFF_FORMATS, SAFETY_STANDARDS, ArcLangCompiler
from .common import (
    MODEL_PATH,
    CommandLine,
    InvalidArguments,
    diff_against,
    report,
    repository_root,
    resolve_path,
    schema,
)

BASELINE_ACTIONS = ("create", "list", "diff", "verify")

DIRECTORY = {
    "type": "string",
    "description": "A directory of the git repository worked on (default: the workspace)",
}

MERGE_SCHEMA = schema(
    {
        "base_path": {"type": "string", "description": "The common ancestor (git's %O)"},
        "ours_path": {"type": "string", "description": "Our version (git's %A)"},
        "theirs_path": {"type": "string", "description": "Their version (git's %B)"},
        "output_path": {
            "type": "string",
            "description": "Write the merged text here (default: nothing is written, the "
            "text is returned)",
        },
        "repository_path": {
            "type": "string",
            "description": "The path of the file in the repository (git's %P, `--path`): "
            "where its imports resolve, and what kind of file it is (.arc or .layout.json)",
        },
        "compile": {
            "type": "boolean",
            "description": "Compile the merged model (default true; false passes --no-compile)",
            "default": True,
        },
    },
    "base_path",
    "ours_path",
    "theirs_path",
)

MERGE_DESCRIPTION = (
    "Three-way merge of a model by element (`arclang merge <base> <ours> <theirs> --json`): "
    "elements are matched by stable identity, so two sides that changed different elements "
    "merge without a conflict even when the text moved; an element changed differently on "
    "both sides is a conflict, "
    "written as markers around the whole element with both complete versions. A "
    "`.layout.json` file is merged entry by entry (an entry changed on both sides takes "
    "theirs, and the answer says so). The merged model is compiled unless `compile: false`. "
    "The first line says clean, the number of conflicts, or that the result does not "
    "compile. Nothing is written without `output_path`."
)

TOOLS: List[Tool] = [
    Tool(name="arclang_merge", description=MERGE_DESCRIPTION, inputSchema=MERGE_SCHEMA),
    Tool(
        name="arclang_git_merge",
        description=(
            "Kept for existing clients: the same tool as arclang_merge, under its first name. "
            + MERGE_DESCRIPTION
        ),
        inputSchema=MERGE_SCHEMA,
    ),
    Tool(
        name="arclang_history",
        description=(
            "The commits that changed one element (`arclang history`), found by the "
            "semantic diff between successive revisions of the model: for each, what "
            "changed of the element (an attribute, a trace, a relationship), not the lines "
            "of the file. The model must be in a git repository."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "element": {"type": "string", "description": "The element: its id or unique name"},
                "limit": {"type": "integer", "description": "The most commits listed (default 20)"},
                "as_json": {"type": "boolean", "description": "The history as JSON", "default": False},
            },
            "model_path",
            "element",
        ),
    ),
    Tool(
        name="arclang_restore",
        description=(
            "The model with some elements, or the whole file, as they were at a git "
            "revision (`arclang restore --rev`): time travel, never a rewrite of history. "
            "With `elements`, only those go back and the rest stays as it is now; an "
            "element removed since is put back in its parent. Nothing is committed. "
            "Without `output_path` the answer is the diff against the file as it is, NOT "
            "written; `output_path` writes the result there (the model itself, to restore "
            "it in place)."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "rev": {
                    "type": "string",
                    "description": "The git revision to go back to (HEAD~3, a tag, baseline/v1)",
                },
                "elements": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Only these elements, by id (default: the whole file)",
                },
                "output_path": {
                    "type": "string",
                    "description": "Write the result here (default: return the diff, write nothing)",
                },
            },
            "model_path",
            "rev",
        ),
    ),
    Tool(
        name="arclang_baseline",
        description=(
            "Baselines of the repository (`arclang baseline`). `action: list` lists them; "
            "`verify` checks that the files of baseline `name` still match its manifest and "
            "its tag; `diff` is the semantic diff between baselines `a` and `b`, with their "
            "gate reports. `create` MAKES one and changes the repository: it checks that "
            "the tree is clean and the model compiles, runs the gate, writes "
            "`baselines/<name>/` (gate report, coverage, the model as JSON and SysML v2, a "
            "manifest) and tags the commit `baseline/<name>` (not pushed); "
            "`require_pass` refuses a model whose gate fails, `allow_dirty` allows "
            "uncommitted changes (the baseline is then not reproducible). Create one only "
            "when the engineer asks for it."
        ),
        inputSchema=schema(
            {
                "action": {
                    "type": "string",
                    "enum": list(BASELINE_ACTIONS),
                    "description": "list, verify, diff, or create (which changes the repository)",
                },
                "directory": DIRECTORY,
                "name": {"type": "string", "description": "create/verify: the baseline's name"},
                "a": {"type": "string", "description": "diff: the first baseline"},
                "b": {"type": "string", "description": "diff: the second baseline"},
                "format": {
                    "type": "string",
                    "enum": list(DIFF_FORMATS),
                    "description": "diff: text, json, md or html",
                    "default": "text",
                },
                "standard": {
                    "type": "string",
                    "enum": list(SAFETY_STANDARDS),
                    "description": "create: the standard of the gate",
                    "default": "iso26262",
                },
                "model_path": {
                    "type": "string",
                    "description": "create: the model file (default: the only .arc file at "
                    "the top of the repository)",
                },
                "require_pass": {
                    "type": "boolean",
                    "description": "create: refuse a model whose gate fails",
                    "default": False,
                },
                "allow_dirty": {
                    "type": "boolean",
                    "description": "create: allow uncommitted changes",
                    "default": False,
                },
                "as_json": {
                    "type": "boolean",
                    "description": "list/verify: the answer as JSON",
                    "default": False,
                },
            },
            "action",
        ),
    ),
]

# The arguments each action of arclang_baseline takes, besides `action` and `directory`.
BASELINE_ARGUMENTS = {
    "create": ("name", "standard", "model_path", "require_pass", "allow_dirty"),
    "list": ("as_json",),
    "diff": ("a", "b", "format"),
    "verify": ("name", "as_json"),
}
BASELINE_REQUIRED = {"create": ("name",), "list": (), "diff": ("a", "b"), "verify": ("name",)}


class VersioningTools:
    """Merge, history, restore, baselines."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_merge": self._merge,
            "arclang_git_merge": self._merge,
            "arclang_history": self._history,
            "arclang_restore": self._restore,
            "arclang_baseline": self._baseline,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown versioning tool: {tool_name}")
        return await handler(arguments)

    def _path(self, args: Dict[str, Any], key: str) -> Path:
        return resolve_path(self.workspace_root, args[key])

    async def _merge(self, args: Dict[str, Any]) -> str:
        base = self._path(args, "base_path")
        ours = self._path(args, "ours_path")
        theirs = self._path(args, "theirs_path")
        line = CommandLine(self.workspace_root, args, "merge", "--json")
        output = line.path("output_path", "--output", make_parent=True)
        line.path("repository_path", "--path")
        if args.get("compile", True) is False:
            line.add("--no-compile")
        run = await self.compiler.run(line.end(base, ours, theirs))
        title = f"Merge of {ours.name} and {theirs.name} over {base.name}"
        try:
            outcome = json.loads(run.stdout)
        except json.JSONDecodeError:
            outcome = None
        if not isinstance(outcome, dict) or "clean" not in outcome:
            # No outcome printed: a file that cannot be read, a text that is no model.
            return report(title, run)
        conflicts = outcome.get("conflicts") or []
        if conflicts:
            verdict = f"{len(conflicts)} CONFLICT(S), written as markers around each element"
        elif outcome.get("compile_error"):
            verdict = "merged without conflict, but THE RESULT DOES NOT COMPILE"
        elif args.get("compile", True) is False:
            verdict = "clean (not compiled: compile: false)"
        else:
            verdict = "clean; the merged model compiles"
        lines = [f"{title}: {verdict}", ""]
        lines += [f"- conflict on {c.get('label')}: {c.get('reason')}" for c in conflicts]
        if outcome.get("compile_error"):
            lines.append(f"The compiler on the merged model: {outcome['compile_error']}")
        lines += [f"- note: {note}" for note in outcome.get("notes") or []]
        text = outcome.get("text")
        if output is not None:
            lines.append(f"Written to {output}")
        elif isinstance(text, str):
            lines += ["Not written (no output_path); the merged text is:", "", text.rstrip()]
        return "\n".join(lines)

    async def _history(self, args: Dict[str, Any]) -> str:
        model = self._path(args, "model_path")
        if args.get("limit") is not None and args["limit"] < 1:
            raise InvalidArguments("'limit' must be at least 1")
        line = CommandLine(self.workspace_root, args, "history").value("limit", "--limit")
        run = await self.compiler.run(line.flag("as_json", "--json").end(model, args["element"]))
        return report(f"History of '{args['element']}' in {model.name}", run)

    async def _restore(self, args: Dict[str, Any]) -> str:
        model = self._path(args, "model_path")
        rev = args["rev"]
        elements = args.get("elements") or []
        what = ", ".join(elements) if elements else "the whole file"
        title = f"Restore of {what} of {model.name} at {rev}"
        line = CommandLine(self.workspace_root, args, "restore", f"--rev={rev}").each("elements", "--element")
        output = line.path("output_path", "--output", make_parent=True)
        run = await self.compiler.run(line.end(model))
        if not run.ok:
            return report(title, run)
        if output is not None:
            return report(title, run, f"written to {output}")
        try:
            before = model.read_text()
        except OSError as failure:
            return f"{title}: cannot read {model}: {failure}"
        answer = diff_against(title, model.name, before, run.stdout)
        return answer.replace("pass write: true to apply", "give output_path to write it")

    async def _baseline(self, args: Dict[str, Any]) -> str:
        action = args["action"]
        allowed = BASELINE_ARGUMENTS[action]
        extra = [key for key in args if key not in ("action", "directory", *allowed)]
        if extra:
            raise InvalidArguments(
                f"{', '.join(repr(key) for key in extra)} mean(s) nothing for 'action: {action}': "
                f"it takes {', '.join(allowed)}"
            )
        missing = [key for key in BASELINE_REQUIRED[action] if not args.get(key)]
        if missing:
            raise InvalidArguments(f"'action: {action}' needs {', '.join(repr(key) for key in missing)}")
        directory = resolve_path(self.workspace_root, args.get("directory") or ".")
        if action == "create":
            # The baseline is written at the top of the repository: that too must be allowed.
            repository_root(self.workspace_root, directory)
        line = CommandLine(self.workspace_root, args, "baseline", action)
        positional: List[str] = []
        if action == "create":
            line.value("standard", "--standard")
            line.path("model_path", "--model")
            line.flag("require_pass", "--require-pass").flag("allow_dirty", "--allow-dirty")
            positional = [args["name"]]
            title = f"Baseline '{args['name']}'"
        elif action == "list":
            line.flag("as_json", "--json")
            title = f"Baselines of {directory.name}"
        elif action == "diff":
            line.value("format", "--format")
            positional = [args["a"], args["b"]]
            title = f"Baseline diff {args['a']} -> {args['b']}"
        else:
            line.flag("as_json", "--json")
            positional = [args["name"]]
            title = f"Verification of baseline '{args['name']}'"
        run = await self.compiler.run(line.end(*positional), cwd=directory)
        verdict = None
        if action == "create" and run.ok:
            verdict = "created (a directory under baselines/ and a tag, not pushed)"
        elif action == "verify":
            verdict = "matches its manifest and its tag" if run.ok else None
            if not run.ok and "FAIL" in run.stdout:
                verdict = "DOES NOT MATCH its manifest or its tag"
        elif action == "diff":
            # Like `arclang diff`: 0 no difference, 1 a difference; anything else did not run.
            if run.returncode == 0:
                verdict = "no semantic change"
            elif run.returncode == 1 and run.stdout.strip():
                return f"{title}: the baselines differ\n\n{run.output}"
        return report(title, run, verdict)
