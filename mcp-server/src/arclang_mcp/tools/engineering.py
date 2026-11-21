"""
Engineering verdicts: a part, a circuit, a chip block or a photonic transform judged
by an engine, and the verdicts read back.

The engines belong to the Enterprise edition: `arclang-server engineering ...`
runs them, and this tool only calls it. The server names no engine: what can
judge, with which tools and whether its engine is built, is what
`connectors` prints. Without `arclang-server` (the `ARCLANG_SERVER` variable,
else the PATH) the answer says the Enterprise edition is needed and runs
nothing. The tool passes no credential: an engine run online is reached with
the environment the server was started with, and nothing of it is printed.

The first line of an answer is the outcome as the server's own output and exit
code say it, never better: a limit that does not hold, a verdict that is stale
or was not run, a limit that could not be judged, each have their word.
"""

import os
import re
import shutil
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..compiler.wrapper import ArcLangCompiler
from .common import MODEL_PATH, InvalidArguments, bounded_seconds, report, resolve_path, schema

SERVER_VARIABLE = "ARCLANG_SERVER"
SERVER_PROGRAM = "arclang-server"

# Seconds a call may take when it does not say: a chip layout is minutes of a big machine.
SECONDS = 900
SECONDS_MAX = 3600

# What each action needs of the call, and what it may carry beyond.
ACTIONS = {
    "connectors": "what can judge, with which tools, and whether its engine is built",
    "model": "judge every part the model realizes by an engine; the verdicts are kept beside the model",
    "status": "the kept verdicts read against the model as it is now; runs nothing",
    "credibility": "the six questions each kept verdict answers, by what ran",
    "artifacts": "what the verdicts rest on and what the runs produced, with hashes",
    "history": "the journal of verdicts, each time a unit's verdicts or key changed",
    "impact": "what a change reaches: units, requirements, cases, artifacts",
    "size": "a parameter of a part tried over a range, each value solved and judged",
    "trade": "the same part under several materials of the registry, each solved and judged",
    "propose-value": "a component of a circuit part tried over the values of a standard series, each simulated and judged",
    "propose": "a part's shape proposed for several shares of material kept, each judged by the solvers; nothing is written",
    "calibrate": "a parameter of a twin's physics fitted over a range to the measurements the twin holds",
}
NEEDS_MODEL = tuple(name for name in ACTIONS if name != "connectors")
# The most values `size` tries, as the server allows.
SIZE_VALUES_MAX = 41
# `propose-value` tries each value of a series in the range: three decades at most (72 values of E24).
SERIES_DECADES_MAX = 1000

NAME = re.compile(r"^[A-Za-z0-9_][A-Za-z0-9_.:+-]*$")
TARGET = re.compile(r"^[A-Za-z0-9_][A-Za-z0-9_.:/+-]*$")
# A git revision: HEAD~2, main^, v1.2.0, a branch.
REVISION = re.compile(r"^[A-Za-z0-9_][A-Za-z0-9_.:/+~^-]*$")

TOOLS: List[Tool] = [
    Tool(
        name="arclang_engineering",
        description=(
            "Judge the parts of a model by the engineering engines of the Enterprise edition "
            "and read the verdicts (`arclang-server engineering`). A node says what realizes it "
            "(`realized_by`) and its limits; each limit comes back pass, FAIL or inconclusive "
            "with what was measured. `action`: connectors (what can judge, which tool, whether "
            "its engine is built: call it first), model (run the engines, keep the verdicts), "
            "status (are the kept verdicts fresh against the model; runs nothing), credibility "
            "(what each verdict rests on: the six questions), artifacts, history, impact (what "
            "a change reaches), size (a parameter over a range), trade (materials compared), "
            "propose-value (a component over a standard series, each value judged), propose (a "
            "shape per share of material kept, each judged; writes nothing), calibrate (a twin's "
            "physics parameter fitted to its measurements; it runs the twin, so the server needs the "
            "open compiler with `arclang twin` (ARCLANG_BIN) and OpenModelica). A proposal or a fit is a candidate: "
            "the limits and a fresh `model` decide, and the model is never changed by it. "
            "Say a limit holds only when the first line says every limit holds; a stale or "
            "missing verdict is not a pass. These are simulations and calculations on a model, "
            "not measurements of a product: say what the credibility answer says and no more. "
            "Needs the Enterprise edition (`arclang-server`); without it the answer says so."
        ),
        inputSchema=schema(
            {
                "action": {"type": "string", "enum": list(ACTIONS), "description": "; ".join(f"{k}: {v}" for k, v in ACTIONS.items())},
                "model_path": {**MODEL_PATH, "description": "The .arc model (every action but connectors)"},
                "id": {"type": "string", "description": "credibility, history: only this unit (PN-...)"},
                "targets": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "impact: what changes: a file, a registry entry, an element or a requirement",
                },
                "since": {"type": "string", "description": "impact: a git revision; what changed since is added to the targets"},
                "part": {"type": "string", "description": "size, trade, propose-value, propose: the part (PN-...)"},
                "parameter": {"type": "string", "description": "size: the parameter sized; calibrate: the parameter of the physics fitted"},
                "from": {"type": "number", "description": "size, propose-value, calibrate: first value (propose-value: above 0)"},
                "to": {"type": "number", "description": "size, propose-value, calibrate: last value"},
                "component": {"type": "string", "description": "propose-value: the component to size (R1 for instance)"},
                "series": {"type": "string", "description": "propose-value: the standard series (default E24)"},
                "volume": {
                    "type": "array",
                    "items": {"type": "number"},
                    "description": "propose: the shares of the part to keep, each from 0.1 to 0.9 (0.7, 0.5, 0.4 for instance)",
                },
                "twin": {"type": "string", "description": "calibrate: the twin (TW-...)"},
                "source": {"type": "string", "description": "calibrate: the physics file, relative to the model, when the twin names none"},
                "step": {"type": "number", "description": f"size: step, above 0; at most {SIZE_VALUES_MAX} values are tried"},
                "materials": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "trade: at least two entries of the registry",
                },
                "timeout": {
                    "type": "integer",
                    "description": f"Seconds the run may take (default {SECONDS}, at most {SECONDS_MAX})",
                    "default": SECONDS,
                },
            },
            "action",
        ),
    ),
]


def find_server() -> Optional[str]:
    """The Enterprise server: ARCLANG_SERVER, else the PATH; None when there is none."""
    given = os.environ.get(SERVER_VARIABLE, "").strip()
    if given:
        return str(Path(given).expanduser())
    return shutil.which(SERVER_PROGRAM)


def name_of(value: Any, what: str, pattern: "re.Pattern[str]" = NAME) -> str:
    """A name the server reads as a word: never an option, never a path out of the model's place."""
    if not isinstance(value, str) or not pattern.match(value) or ".." in value.split("/"):
        raise InvalidArguments(f"'{what}' is a name (letters, digits, - _ . : +), not an option or a path outside the model")
    return value


def number_of(args: Dict[str, Any], key: str) -> float:
    value = args.get(key)
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise InvalidArguments(f"'{key}' is a number")
    return float(value)


def only(args: Dict[str, Any], action: str, *allowed: str) -> None:
    """Refuse the arguments that mean nothing for the action, rather than drop them."""
    extra = [key for key in args if key not in ("action", "model_path", "timeout", *allowed)]
    if extra:
        raise InvalidArguments(f"{', '.join(repr(k) for k in extra)} mean(s) nothing for '{action}'")


def outcome(action: str, ran_ok: bool, text: str) -> Optional[str]:
    """The first-line verdict of `model` and `status`, read from what the server printed."""
    if action not in ("model", "status"):
        return None
    if "✗ FAIL" in text:
        return "A LIMIT DOES NOT HOLD"
    if "NOT RUN" in text:
        return "NOT RUN (a part has no kept verdict; this is not a pass)"
    if "STALE" in text:
        return "STALE (what it was run on has changed; this is not a pass)"
    if "? inconclusive" in text:
        return "NOT JUDGED (a limit is inconclusive; this is not a pass)"
    if ran_ok:
        return "every limit holds" if action == "model" else "every verdict is fresh and passes"
    return None


class EngineeringTools:
    """The engineering engines, through `arclang-server engineering`."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_engineering": self._engineering,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown engineering tool: {tool_name}")
        return await handler(arguments)

    def _arguments(self, args: Dict[str, Any]) -> List[str]:
        """The words after `engineering`, checked: an option is never a value."""
        action = args.get("action")
        if action not in ACTIONS:
            raise InvalidArguments(f"'action' is one of {', '.join(ACTIONS)}")
        if action == "connectors":
            only(args, action)
            if args.get("model_path") is not None:
                raise InvalidArguments("'connectors' takes no 'model_path': it needs no model")
            return [action]
        if not args.get("model_path"):
            raise InvalidArguments(f"'{action}' needs 'model_path'")
        words = [action, str(resolve_path(self.workspace_root, args["model_path"]))]
        if action in ("credibility", "history"):
            only(args, action, "id")
            if args.get("id") is not None:
                words.append(name_of(args["id"], "id"))
        elif action == "impact":
            only(args, action, "targets", "since")
            targets = args.get("targets") or []
            if not isinstance(targets, list):
                raise InvalidArguments("'targets' is a list")
            words += [name_of(t, "targets", TARGET) for t in targets]
            if args.get("since") is not None:
                words += ["--since", name_of(args["since"], "since", REVISION)]
            elif not targets:
                raise InvalidArguments("name what changes in 'targets', or give 'since'")
        elif action == "size":
            only(args, action, "part", "parameter", "from", "to", "step")
            low, high, step = (number_of(args, key) for key in ("from", "to", "step"))
            if step <= 0 or high < low or (high - low) / step > SIZE_VALUES_MAX - 1:
                raise InvalidArguments(f"'from' is under 'to', 'step' is above 0, and at most {SIZE_VALUES_MAX} values are tried")
            words += [
                "--part", name_of(args.get("part"), "part"),
                "--parameter", name_of(args.get("parameter"), "parameter"),
                "--from", f"{low:g}", "--to", f"{high:g}", "--step", f"{step:g}",
            ]
        elif action == "trade":
            only(args, action, "part", "materials")
            materials = args.get("materials")
            if not isinstance(materials, list) or len(materials) < 2:
                raise InvalidArguments("'materials' lists at least two entries of the registry")
            if any("," in str(m) for m in materials):
                raise InvalidArguments("an entry of 'materials' must not hold a comma")
            words += [
                "--part", name_of(args.get("part"), "part"),
                "--materials", ",".join(name_of(m, "materials") for m in materials),
            ]
        elif action == "propose-value":
            only(args, action, "part", "component", "series", "from", "to")
            low, high = number_of(args, "from"), number_of(args, "to")
            if low <= 0 or high < low or high / low > SERIES_DECADES_MAX:
                raise InvalidArguments(f"'from' is above 0 and under 'to', and 'to' is at most {SERIES_DECADES_MAX} times 'from'")
            words += [
                "--part", name_of(args.get("part"), "part"),
                "--component", name_of(args.get("component"), "component"),
                "--series", name_of(args.get("series", "E24"), "series"),
                "--from", f"{low:g}", "--to", f"{high:g}",
            ]
        elif action == "propose":
            only(args, action, "part", "volume")
            shares = args.get("volume")
            if not isinstance(shares, list) or not 1 <= len(shares) <= 8 or any(isinstance(v, bool) or not isinstance(v, (int, float)) or not 0.1 <= v <= 0.9 for v in shares):
                raise InvalidArguments("'volume' lists one to eight shares, each from 0.1 to 0.9")
            words += ["--part", name_of(args.get("part"), "part"), "--volume", ",".join(f"{v:g}" for v in shares)]
        elif action == "calibrate":
            only(args, action, "twin", "parameter", "from", "to", "source")
            low, high = number_of(args, "from"), number_of(args, "to")
            if high <= low:
                raise InvalidArguments("'from' is under 'to'")
            words += [
                "--twin", name_of(args.get("twin"), "twin"),
                "--parameter", name_of(args.get("parameter"), "parameter"),
                "--from", f"{low:g}", "--to", f"{high:g}",
            ]
            if args.get("source") is not None:
                words += ["--source", name_of(args["source"], "source", TARGET)]
        else:
            only(args, action)
        return words

    async def _engineering(self, args: Dict[str, Any]) -> str:
        words = self._arguments(args)
        action = words[0]
        title = f"Engineering {action}" + (f" of {Path(words[1]).name}" if action in NEEDS_MODEL else "")
        server = find_server()
        if server is None:
            return (
                f"{title}: UNAVAILABLE (the Enterprise edition is needed)\n\n"
                f"`{SERVER_PROGRAM}` was not found: the engineering engines belong to the "
                f"Enterprise edition. Install it, or point {SERVER_VARIABLE} at it. Nothing was run, "
                "and no verdict is known."
            )
        limit = bounded_seconds(args, "timeout", SECONDS, SECONDS_MAX)
        run = await self.compiler.run(["engineering", *words], timeout=limit, binary=server)
        return report(title, run, outcome(action, run.ok, run.output))
