"""
Reverse engineering: the model a piece of code implements, the code as built
against the model, and the proof that code written by hand is a machine of
the model. Each tool is a subcommand of the compiler and returns what it
printed; none of them says "proven" unless the compiler did.
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List

from mcp.types import Tool

from ..compiler.wrapper import ArcLangCompiler
from .common import MODEL_PATH, InvalidArguments, report, resolve_path, schema

# The time one proof may take when the call does not say, and the most a call may ask.
PROOF_SECONDS = 300
PROOF_SECONDS_MAX = 3600

TIMEOUT = {
    "type": "integer",
    "description": f"Seconds one proof may take (default {PROOF_SECONDS}, at most {PROOF_SECONDS_MAX})",
    "default": PROOF_SECONDS,
}

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
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_recover",
        description=(
            "Read the state machines back from the C the compiler generated for a model "
            "(`arclang recover`): `code_path` is the directory of `arclang export -f c`, or "
            "its `<model>.h`. Returns the machines as ArcLang. With `check: true` the "
            "recovered model must generate that same C, line for line: only then is the "
            "recovery proven. `hdl_path` (the directory of `arclang export -f vhdl`) sizes "
            "the machines the logic computes and, with `check`, proves the logic too. For "
            "code written by hand use arclang_prove. When `code_path` is a flattened device "
            "tree (`.dts`: `dtc -I fs -O dts /proc/device-tree` on the board), the answer is "
            "the board as a physical architecture: its devices, the bus each sits on, the "
            "pins that wire them, as the tree says them."
        ),
        inputSchema=schema(
            {
                "code_path": {
                    "type": "string",
                    "description": "The directory of the generated C, or its header",
                },
                "check": {
                    "type": "boolean",
                    "description": "Prove the recovery by generating the code again",
                    "default": True,
                },
                "hdl_path": {
                    "type": "string",
                    "description": "The directory of the generated VHDL",
                },
                "output_path": {
                    "type": "string",
                    "description": "Write the recovered model to this file (returned otherwise)",
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
        arguments = ["prove", "--timeout", str(seconds)]
        if args.get("correspondence"):
            arguments += ["--correspondence", str(args["correspondence"])]
        arguments += ["--", str(model)]
        # Two obligations, and each line asked again when one is refuted.
        run = await self.compiler.run(arguments, timeout=seconds * 4 + 120)
        verdict = None
        if "✗ Refuted" in run.output:
            verdict = "REFUTED"
        elif "Neither proven nor refuted" in run.output:
            verdict = "NOT PROVEN (neither proven nor refuted)"
        elif run.ok and "✓ Proven" in run.output:
            verdict = "proven"
        return report(f"Proof of the correspondences of {model.name}", run, verdict)

    async def _recover(self, args: Dict[str, Any]) -> str:
        code = self._path(args, "code_path")
        if code.suffix == ".dts":
            # A board: its device tree is read, and nothing of it is to prove.
            arguments = ["recover"]
            if args.get("output_path"):
                arguments += ["--output", str(self._path(args, "output_path"))]
            run = await self.compiler.run(arguments + ["--", str(code)])
            verdict = "read from the device tree (read, not proven)" if run.ok else None
            return report(f"Board of {code.name}", run, verdict)
        check = args.get("check", True)
        arguments = ["recover"]
        if check:
            arguments.append("--check")
        if args.get("hdl_path"):
            arguments += ["--hdl", str(self._path(args, "hdl_path"))]
        if args.get("output_path"):
            arguments += ["--output", str(self._path(args, "output_path"))]
        arguments += ["--", str(code)]
        run = await self.compiler.run(arguments)
        verdict = None
        if run.ok:
            verdict = "recovered and proven" if check else "recovered, NOT proven (check: false)"
        return report(f"Recovery of the machines of {code.name}", run, verdict)

    async def _drift(self, args: Dict[str, Any]) -> str:
        model = self._path(args, "model_path")
        code = self._path(args, "code_path")
        arguments = ["drift"]
        if args.get("hdl_path"):
            arguments += ["--hdl", str(self._path(args, "hdl_path"))]
        limit = None
        if args.get("prove"):
            seconds = proof_seconds(args)
            arguments += ["--prove", "--timeout", str(seconds)]
            limit = seconds * 4 + 120
        arguments += ["--", str(model), str(code)]
        run = await self.compiler.run(arguments, timeout=limit)
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
