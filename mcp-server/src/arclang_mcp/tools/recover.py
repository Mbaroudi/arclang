"""
Reverse engineering: the model a piece of code implements, the code as built
against the model, and the proof that code written by hand is a machine of
the model. Each tool is a subcommand of the compiler and returns what it
printed; none of them says "proven" unless the compiler did: a device tree
and a binary are read, a proposal is proven before it is written.
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import ArcLangCompiler
from .common import (
    MODEL_PATH,
    CommandLine,
    InvalidArguments,
    command_argument,
    report,
    resolve_path,
    schema,
)

# The time one proof may take when the call does not say, and the most a call may ask.
PROOF_SECONDS = 300
PROOF_SECONDS_MAX = 3600

TIMEOUT = {
    "type": "integer",
    "description": f"Seconds one proof may take (default {PROOF_SECONDS}, at most {PROOF_SECONDS_MAX})",
    "default": PROOF_SECONDS,
}

# The turns a proposer gets when the call does not say, and the most a call may ask.
PROPOSER_TURNS_MAX = 12
# One answer of a proposer may take this long (PROPOSER_SECONDS in src/cli/propose.rs).
PROPOSER_SECONDS = 600

TOOLS: List[Tool] = [
    Tool(
        name="arclang_prove",
        description=(
            "Prove that code written by hand (an Arduino sketch) is a state machine of the "
            "model, as each `correspondence` of the model says (`arclang prove`). The answer "
            "is `Proven`, or `Refuted` with the lines of the correspondence that do not hold "
            "and a case that shows it (the inputs, both sides of each line before and after "
            "the step), or `Neither proven nor refuted` with the reason. To recover the model "
            "of a sketch yourself: write the state_machine and the correspondence beside the "
            "sketch, call this tool, read what is refuted, correct, call again; say the model "
            "is recovered only once this tool answers Proven. Needs CBMC and Bitwuzla "
            "(ArcLang's formal image); without them the answer says so."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "correspondence": {
                    "type": "string",
                    "description": "Only this correspondence (default: every one of the model)",
                },
                "timeout": TIMEOUT,
                "as_json": {
                    "type": "boolean",
                    "description": "The proof as JSON (`--json`): each obligation and its outcome",
                    "default": False,
                },
                "keep_dir": {
                    "type": "string",
                    "description": "Keep the harnesses and the provers' files in this directory",
                },
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_recover",
        description=(
            "Read a model back from what was built (`arclang recover`). What `code_path` is "
            "decides what is read, and what the answer may claim. (1) The directory of "
            "`arclang export -f c`, or its `<model>.h`: the state machines, as ArcLang; with "
            "`check: true` the recovered model must generate that same C, line for line: "
            "only then is the recovery proven. `hdl_path` (the directory of `arclang export "
            "-f vhdl`) sizes the machines the logic computes and, with `check`, proves the "
            "logic too; `view_path` draws the recovered model in an HTML page (written, "
            "never opened). (2) A flattened device tree (`.dts`: `dtc -I fs -O dts "
            "/proc/device-tree` on the board): the board as a physical architecture, its "
            "devices, the bus each sits on, the pins that wire them. Read, not proven. (3) A "
            "firmware binary (`.hex`, Intel HEX of an ATmega328P): its machine code is read "
            "with Ghidra (ArcLang's reverse image) and a correspondence is PROPOSED: where "
            "its loop or cycle is, what was found and what was not. Nothing is proven: write "
            "the machine, then arclang_prove decides. `model_path` names the model the "
            "binary is said to be built from: its step is then looked for in the binary. "
            "(4) With `propose: true`, a sketch written by hand (`.ino`) or a `.hex`: a "
            "proposer (an AI behind a command) proposes the state machine and the "
            "correspondence, ArcLang proves each proposal as arclang_prove does and tells "
            "the proposer what failed, for `turns` turns; a model is written only once its "
            "proof holds. PRIVACY: the sketch is sent to the proposer's command. The "
            "proposer is ARCLANG_PROPOSER of the server's environment; `with` names another "
            "one only when the server was started with ARCLANG_MCP_ALLOW_COMMANDS=1. To "
            "recover a sketch yourself without a proposer, write the machine and the "
            "correspondence and call arclang_prove."
        ),
        inputSchema=schema(
            {
                "code_path": {
                    "type": "string",
                    "description": "The directory of the generated C or its header; a .dts "
                    "device tree; a .hex firmware; with propose, a .ino sketch or a .hex",
                },
                "check": {
                    "type": "boolean",
                    "description": "Generated C only: prove the recovery by generating the "
                    "code again",
                    "default": True,
                },
                "hdl_path": {
                    "type": "string",
                    "description": "The directory of the generated VHDL",
                },
                "output_path": {
                    "type": "string",
                    "description": "Write the recovered model to this file (returned otherwise; "
                    "with propose, default: beside the sketch)",
                },
                "model_path": {
                    "type": "string",
                    "description": ".hex only: the model the binary is said to be built from",
                },
                "propose": {
                    "type": "boolean",
                    "description": "Have a proposer propose the model, proven before it is written",
                    "default": False,
                },
                "with": {
                    "type": "string",
                    "description": "With propose: the proposer's command (the prompt on its "
                    "input, its answer on its output). Needs ARCLANG_MCP_ALLOW_COMMANDS=1",
                },
                "turns": {
                    "type": "integer",
                    "description": f"With propose: turns the proposer gets (default 4, at most {PROPOSER_TURNS_MAX})",
                },
                "timeout": TIMEOUT,
                "view_path": {
                    "type": "string",
                    "description": "Draw the recovered (generated C) or proven (propose) model "
                    "in this HTML page",
                },
                "keep_dir": {
                    "type": "string",
                    "description": ".hex or propose: keep the working files (the lifted code, "
                    "each turn's answer) in this directory",
                },
            },
            "code_path",
        ),
    ),
    Tool(
        name="arclang_drift",
        description=(
            "Compare the code as built with the model as designed (`arclang drift`): each "
            "difference of the machines (a guard, an initial value, an equation, the time or "
            "the target of a transition) in the terms of the model with its line in the "
            "code, and every other generated file line by line (`hdl_path` adds the VHDL). "
            "With `prove: true` the code as built is proven to do what the model's code "
            "does, or refuted with the state and the inputs (C: CBMC, Bitwuzla) or the clock "
            "and the outputs (VHDL: GHDL, Yosys, SymbiYosys) that show it; a proof that does "
            "not end is reported as neither. Say the code matches the model only when the "
            "answer says no drift or proven equivalent."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "code_path": {
                    "type": "string",
                    "description": "The directory of the C as built, or its header",
                },
                "hdl_path": {
                    "type": "string",
                    "description": "The directory of the VHDL as built",
                },
                "prove": {
                    "type": "boolean",
                    "description": "Prove or refute that the code as built does the same",
                    "default": False,
                },
                "timeout": TIMEOUT,
                "as_json": {
                    "type": "boolean",
                    "description": "The differences as JSON (`--json`)",
                    "default": False,
                },
            },
            "model_path",
            "code_path",
        ),
    ),
]


def proof_seconds(args: Dict[str, Any]) -> int:
    """The time a proof may take, as the call gives it."""
    seconds = args.get("timeout", PROOF_SECONDS)
    if isinstance(seconds, bool) or not isinstance(seconds, int) or not 1 <= seconds <= PROOF_SECONDS_MAX:
        raise InvalidArguments(f"'timeout' is a number of seconds from 1 to {PROOF_SECONDS_MAX}")
    return seconds


class RecoverTools:
    """Prove, recover, drift."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_prove": self._prove,
            "arclang_recover": self._recover,
            "arclang_drift": self._drift,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown recover tool: {tool_name}")
        return await handler(arguments)

    def _path(self, args: Dict[str, Any], key: str) -> Path:
        return resolve_path(self.workspace_root, args[key])

    async def _prove(self, args: Dict[str, Any]) -> str:
        model = self._path(args, "model_path")
        seconds = proof_seconds(args)
        line = CommandLine(self.workspace_root, args, "prove", f"--timeout={seconds}")
        line.value("correspondence", "--correspondence").flag("as_json", "--json")
        line.path("keep_dir", "--keep")
        # Two obligations, and each line asked again when one is refuted.
        run = await self.compiler.run(line.end(model), timeout=seconds * 4 + 120)
        verdict = None
        if "✗ Refuted" in run.output:
            verdict = "REFUTED"
        elif "Neither proven nor refuted" in run.output:
            verdict = "NOT PROVEN (neither proven nor refuted)"
        elif run.ok and "✓ Proven" in run.output:
            verdict = "proven"
        elif run.ok and args.get("as_json", False):
            # The JSON says each outcome: the exit code alone is not turned into a word.
            verdict = "ran; read the outcome of each obligation in the JSON"
        return report(f"Proof of the correspondences of {model.name}", run, verdict)

    async def _recover(self, args: Dict[str, Any]) -> str:
        code = self._path(args, "code_path")
        if args.get("propose", False):
            return await self._propose(code, args)
        for key in ("with", "turns"):
            if args.get(key) is not None:
                raise InvalidArguments(f"'{key}' goes with 'propose: true'")
        line = CommandLine(self.workspace_root, args, "recover")
        if code.suffix == ".dts":
            # A board: its device tree is read, and nothing of it is to prove.
            self._only(args, "a device tree", "output_path")
            line.path("output_path", "--output")
            run = await self.compiler.run(line.end(code))
            verdict = "read from the device tree (read, not proven)" if run.ok else None
            return report(f"Board of {code.name}", run, verdict)
        if code.suffix == ".hex":
            # A binary: read with Ghidra, a correspondence proposed. `--check` proves
            # nothing of it, so it is not passed and nothing is called proven.
            self._only(args, "a binary", "output_path", "model_path", "keep_dir", "timeout")
            line.path("output_path", "--output")
            line.path("model_path", "--model")
            line.path("keep_dir", "--keep")
            run = await self.compiler.run(line.end(code), timeout=proof_seconds(args))
            verdict = None
            if run.ok and "the step of the model is the function at" in run.output:
                verdict = "the step of the model placed in the binary (a proposal, NOT proven)"
            elif run.ok:
                verdict = "read from the binary, a correspondence proposed (NOT proven)"
            return report(f"Reading of the binary {code.name}", run, verdict)
        self._only(args, "generated C", "check", "hdl_path", "output_path", "view_path")
        check = args.get("check", True)
        if check:
            line.add("--check")
        line.path("hdl_path", "--hdl")
        line.path("output_path", "--output")
        line.path("view_path", "--view", make_parent=True)
        run = await self.compiler.run(line.end(code))
        verdict = None
        if run.ok:
            verdict = "recovered and proven" if check else "recovered, NOT proven (check: false)"
        return report(f"Recovery of the machines of {code.name}", run, verdict)

    @staticmethod
    def _only(args: Dict[str, Any], what: str, *allowed: str) -> None:
        """Refuse the arguments that mean nothing for what is read, rather than drop them."""
        extra = [
            key
            for key in args
            if key not in ("code_path", "propose", *allowed)
            and not (key == "check" and args[key] is True)
        ]
        if extra:
            raise InvalidArguments(
                f"{', '.join(repr(key) for key in extra)} mean(s) nothing for {what}: "
                f"it takes {', '.join(allowed)}"
            )

    async def _propose(self, code: Path, args: Dict[str, Any]) -> str:
        """`recover --propose`: a proposer proposes, the proof decides, for a bounded time."""
        self._only(args, "a proposal", "with", "turns", "timeout", "output_path", "view_path", "keep_dir")
        seconds = proof_seconds(args)
        turns = args.get("turns", 4)
        if not 1 <= turns <= PROPOSER_TURNS_MAX:
            raise InvalidArguments(f"'turns' is from 1 to {PROPOSER_TURNS_MAX}")
        line = CommandLine(
            self.workspace_root, args, "recover", "--propose", f"--turns={turns}", f"--timeout={seconds}"
        )
        proposer = command_argument(
            args,
            "with",
            "Without it the proposer is ARCLANG_PROPOSER, as the server's environment sets it.",
        )
        if proposer is not None:
            line.add(f"--with={proposer}")
        line.path("output_path", "--output", make_parent=True)
        line.path("view_path", "--view", make_parent=True)
        line.path("keep_dir", "--keep")
        # Each turn: one answer of the proposer, then the two obligations of a proof.
        limit = turns * (PROPOSER_SECONDS + seconds * 4) + 120
        run = await self.compiler.run(line.end(code), timeout=limit)
        verdict = None
        if run.ok and "✓ Proven" in run.output:
            verdict = "proposed, then proven"
        elif "was proven in" in run.output:
            verdict = "NOT PROVEN: no proposal was proven, nothing is written"
        return report(f"Proposal of a model of {code.name}", run, verdict)

    async def _drift(self, args: Dict[str, Any]) -> str:
        model = self._path(args, "model_path")
        code = self._path(args, "code_path")
        line = CommandLine(self.workspace_root, args, "drift").flag("as_json", "--json")
        line.path("hdl_path", "--hdl")
        limit = None
        if args.get("prove"):
            seconds = proof_seconds(args)
            line.add("--prove", f"--timeout={seconds}")
            limit = seconds * 4 + 120
        run = await self.compiler.run(line.end(model, code), timeout=limit)
        if args.get("as_json", False):
            # The JSON says what differs; only an exit code is read into the first line.
            verdict = "no drift" if run.ok else ("DRIFT" if run.stdout.lstrip().startswith("{") else None)
            return report(f"Drift of {code.name} against {model.name}", run, verdict)
        verdict = None
        if "Not equivalent" in run.output:
            verdict = "NOT EQUIVALENT"
        elif run.ok and "Proven equivalent" in run.output:
            verdict = "no drift, or proven equivalent"
        elif run.ok:
            verdict = "no drift"
        elif "has drifted" in run.output:
            verdict = "DRIFT"
        return report(f"Drift of {code.name} against {model.name}", run, verdict)
