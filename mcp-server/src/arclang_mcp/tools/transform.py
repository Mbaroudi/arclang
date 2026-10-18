"""
Transform tools: read a foreign file into an ArcLang model, and declare the
next Arcadia level from the current one.

Both are the compiler's own commands (`arclang import`, `arclang transition`)
and both produce ArcLang: the import writes where the call names an
`output_path` and nothing else; the transition prints the edited model and
writes it back only with `write: true`. The answer relays what the compiler
counted, left out or could not resolve, and never more.
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import IMPORT_FORMATS, TRANSITION_STEPS, ArcLangCompiler, parse_plan
from .common import MODEL_PATH, InvalidArguments, preview, report, resolve_path, schema
from .core import check_verdict

TOOLS: List[Tool] = [
    Tool(
        name="arclang_import",
        description=(
            "Read a foreign file into an ArcLang model (`arclang import -f <format>`) and "
            "write the .arc file at `output_path`. `sysml` reads a SysML v2 text (.sysml, "
            "the dialect ArcLang exports) or a JSON listing of Systems Modeling API records; "
            "`capella` a .capella project; `req-if` and `doors` requirements; `json`, "
            "`yaml`, `xml` the compiled forms; `mermaid` and `plant-uml` diagrams. The "
            "answer is the compiler's account: what it counted, what it left out or left "
            "unresolved, followed by one line on whether the written model compiles "
            "(`arclang check --lint` on it)."
        ),
        inputSchema=schema(
            {
                "input_path": {"type": "string", "description": "The file to read"},
                "format": {
                    "type": "string",
                    "enum": list(IMPORT_FORMATS),
                    "description": "Format of the input file",
                },
                "output_path": {
                    "type": "string",
                    "description": (
                        "The .arc file to write (required: nothing is written elsewhere)"
                    ),
                },
            },
            "input_path",
            "format",
            "output_path",
        ),
    ),
    Tool(
        name="arclang_transition",
        description=(
            "Arcadia transition (`arclang transition`): declare on the next level what "
            "realizes the elements of this one. Steps: oa-to-sa (operational activities to "
            "system functions), sa-to-la (system functions to logical functions), la-to-pa "
            "(logical components to behaviour components of a physical node). Without a "
            "`step`, or with `plan: true`, the answer is the compiler's JSON plan: for each "
            "step, the candidates and what already realizes them; nothing changes. With a "
            "`step` the compiler prints the edited model: the answer is a diff, NOT written "
            "unless `write: true`. `into` names the logical component (sa-to-la) or physical "
            "node (la-to-pa) that receives the created elements; without it the compiler "
            "needs a named `logical_architecture`/`physical_architecture` block to create a "
            "container in, and refuses otherwise. A step with nothing left to realize "
            "changes nothing."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "step": {
                    "type": "string",
                    "enum": list(TRANSITION_STEPS),
                    "description": "The transition to perform (omitted: plan every step)",
                },
                "into": {
                    "type": "string",
                    "description": "Id of the component or node that receives the created elements",
                },
                "plan": {
                    "type": "boolean",
                    "description": "Only say what the step would create, as JSON; change nothing",
                    "default": False,
                },
                "write": {
                    "type": "boolean",
                    "description": (
                        "Write the edited model back. Default: return a diff and change nothing"
                    ),
                    "default": False,
                },
            },
            "model_path",
        ),
    ),
]


class TransformTools:
    """Import and Arcadia transitions."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_import": self._import,
            "arclang_transition": self._transition,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a transform tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown transform tool: {tool_name}")
        return await handler(arguments)

    async def _import(self, args: Dict[str, Any]) -> str:
        source = resolve_path(self.workspace_root, args["input_path"])
        output = resolve_path(self.workspace_root, args["output_path"])
        import_format = args["format"]
        if import_format not in IMPORT_FORMATS:
            return f"Unknown import format '{import_format}'. Known: {', '.join(IMPORT_FORMATS)}"
        output.parent.mkdir(parents=True, exist_ok=True)
        run = await self.compiler.import_model(source, import_format, output)
        title = f"Import of {source.name} ({import_format}) to {output.name}"
        if not run.ok or not output.is_file():
            return report(title, run)
        # The import is the compiler's; so is the word on whether its result compiles.
        check = await self.compiler.check(output)
        verdict = check_verdict(check)
        if verdict is None:
            compiled = f"THE WRITTEN MODEL DOES NOT COMPILE ({output}):\n\n{check.output}"
        else:
            compiled = f"The written model ({output}) {verdict}."
        return f"{report(title, run)}\n\n{compiled}"

    async def _transition(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["model_path"])
        step = args.get("step")
        into = args.get("into")
        write = args.get("write", False)
        plan = args.get("plan", False) or not step
        if write and plan:
            raise InvalidArguments(
                "invalid arguments for arclang_transition: 'write' needs a 'step' and no 'plan'; "
                "a plan changes nothing"
            )
        if plan:
            return await self._plan(model, step, into)
        run = await self.compiler.transition(model, step=step, into=into, write=write)
        title = f"Transition {step} of {model.name}" + (f" into {into}" if into else "")
        if not write:
            answer = preview(title, model, run)
        else:
            verdict = None
            if run.ok and "nothing to do" in run.output:
                verdict = "nothing to do, nothing written"
            elif run.ok and run.output.rstrip().endswith("written"):
                verdict = f"written to {model.name}"
            answer = report(title, run, verdict)
        # What the compiler created, in its own words (`+ ...` lines), when it says so.
        created = [line for line in run.stderr.splitlines() if line.startswith("+ ")]
        if created and not write:
            answer += "\n\nThe compiler declared:\n" + "\n".join(created)
        return answer

    async def _plan(self, model: Path, step: Any, into: Any) -> str:
        run = await self.compiler.transition(model, step=step, into=into, plan=True)
        title = f"Transition plan of {model.name}" + (f" ({step})" if step else "")
        steps = parse_plan(run)
        if steps is None:
            return report(title, run)
        # The summary is read from the compiler's JSON: candidates per step,
        # and how many of them nothing realizes yet.
        summary = []
        for entry in steps:
            candidates = entry.get("candidates", [])
            pending = sum(1 for candidate in candidates if not candidate.get("realized_by"))
            blocked = f", blocked: {entry['blocked']}" if entry.get("blocked") else ""
            summary.append(
                f"{entry.get('step', '?')}: {len(candidates)} candidate(s), "
                f"{pending} not realized{blocked}"
            )
        return f"{title}: {'; '.join(summary) or 'no step'}\n\n{run.stdout.rstrip()}"
