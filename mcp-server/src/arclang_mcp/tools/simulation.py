"""
Running the model: simulation, the stages of the realization ladder, the
digital twins, a board observed, and the step-by-step run of a behavior.

Each tool is one subcommand of the compiler (`arclang_<command>` runs
`arclang <command>`) and returns what it printed. The verdicts are the
compiler's: the first line is read from its exit code, and from nothing the
compiler did not say. The stages build, emulate and simulate with tools the
compiler finds or runs in Docker (OpenModelica, a C compiler, Renode, GHDL,
Yosys, Verilator): a call may take minutes, and takes a `timeout`. Without
its tools a stage fails and says which one is missing.

Three kinds of argument are not model text and are held to more than a
path is: a serial port is a device name (`/dev/...`, `COM3`); a C compiler
is the bare name of a program on the PATH; a command line (`hil`'s
stand-in for the board) is passed on only when the person who started the
server allowed it (ARCLANG_MCP_ALLOW_COMMANDS=1).
"""

from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional

from mcp.types import Tool

from ..compiler.wrapper import STATISTICS_ENGINES, ArcLangCompiler
from .common import (
    MODEL_PATH,
    CommandLine,
    InvalidArguments,
    bounded_seconds,
    command_argument,
    program_name,
    report,
    resolve_path,
    schema,
    serial_port,
)
from .core import RUN_SECONDS, RUN_SECONDS_MAX, RUN_TIMEOUT

# The longest a call may follow live data: a call that never ends answers nothing.
LIVE_SECONDS_MAX = 3600

TEST = {"type": "string", "description": "Only this test case (id or name)"}
AS_JSON = {
    "type": "boolean",
    "description": "The result as JSON (`--json`): each case, its expectations, its verdict",
    "default": False,
}
KEEP_DIR = {
    "type": "string",
    "description": "Keep the generated files, the builds and the logs in this directory",
}

TOOLS: List[Tool] = [
    Tool(
        name="arclang_verify",
        description=(
            "Verification by simulation (`arclang verify`): run the test cases that "
            "`simulate:` in OpenModelica (a local omc, else Docker), check their `expects:` "
            "and cache the verdicts for arclang_validate, arclang_gate and the tables. A "
            "run takes minutes. Without simulating: `list: true` lists the cases that "
            "simulate and whether results of their last run are kept; `check_tools: true` "
            "reports what is installed to run OpenModelica (no model needed); `replay` "
            "prints the results kept of the last run of a case (`point`: the point of a "
            "sweep); `statistics` (r or python) analyzes the samples of the last run of a "
            "Monte Carlo case named by `test`: failure probability by a normal fit, "
            "influence of each parameter (the verdict stays the compiler's); `drawings: "
            "true` lists the Modelica classes worth drawing and `drawing` prints "
            "OpenModelica's instance of one."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "as_json": AS_JSON,
                "list": {
                    "type": "boolean",
                    "description": "Do not simulate: list the cases that simulate, as JSON",
                    "default": False,
                },
                "check_tools": {
                    "type": "boolean",
                    "description": "Do not simulate: report what is installed to run OpenModelica",
                    "default": False,
                },
                "replay": {
                    "type": "string",
                    "description": "Do not simulate: the results kept of the last run of this case",
                },
                "point": {
                    "type": "integer",
                    "description": "With replay: the point of the sweep (0 is the first)",
                },
                "statistics": {
                    "type": "string",
                    "enum": list(STATISTICS_ENGINES),
                    "description": "Do not simulate: analyze the last Monte Carlo run of `test` "
                    "with R or Python",
                },
                "drawings": {
                    "type": "boolean",
                    "description": "Do not simulate: list the Modelica classes worth drawing",
                    "default": False,
                },
                "drawing": {
                    "type": "string",
                    "description": "Do not simulate: OpenModelica's instance of this Modelica class",
                },
                "timeout": RUN_TIMEOUT,
            }
        ),
    ),
    Tool(
        name="arclang_cosim",
        description=(
            "Co-simulation (`arclang cosim`): each test case that `execute:`s, run as a "
            "standard FMI co-simulation in OMSimulator: the behavior's FMU, the plant's and "
            "the case's inputs connected in an SSP package; the run is judged as SIL judges "
            "the C, its `expects:` on it, step by step against the model's own loop. "
            "Without OMSimulator here the compiler runs it in ArcLang Studio (Docker)."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "cc": {
                    "type": "string",
                    "description": "The C compiler the behavior's FMU is built with: the bare "
                    "name of a program on the PATH (default: $CC, else cc)",
                },
                "keep_dir": KEEP_DIR,
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_sil",
        description=(
            "Software in the loop (`arclang sil`): compile the C generated from the state "
            "machines on this computer, run each test case that `execute:`s with its input "
            "vectors, compare the outputs with the model-level run step by step and judge "
            "its `expects:` on both. Needs a C compiler on the PATH."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "cc": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "The C compilers, each the bare name of a program on the "
                    "PATH (gcc, clang, gcc-15): every case runs with each (default: $CC, else cc)",
                },
                "keep_dir": KEEP_DIR,
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_pil",
        description=(
            "Processor in the loop (`arclang pil`): build the generated C for a target "
            "package (Zephyr, `zephyr/litex_vexriscv`), run it on the target's processor in "
            "its emulator (Renode), drive each test case that `execute:`s through the bound "
            "channels, compare the outputs with the model-level run step by step and judge "
            "its `expects:` on both. Needs ArcLang's embedded image (Docker)."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "target": {
                    "type": "string",
                    "description": "The target package (default: the io_binding's `target:`)",
                },
                "keep_dir": KEEP_DIR,
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_hdl",
        description=(
            "Hardware in simulation (`arclang hdl`): the VHDL of each state machine "
            "allocated to programmable logic simulated by GHDL through its registers, each "
            "test case that `execute:`s compared step by step with the bit-exact fixed-point "
            "C (difference 0) and with the model (within the quantisation tolerance), its "
            "`expects:` judged. `synth: true` also synthesises, places and routes each block "
            "for its node's ECP5 (Yosys, nextpnr-ecp5) and reports its resources and Fmax; "
            "`as_if` every machine as if allocated to a part at a clock; `machine` only one."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "synth": {
                    "type": "boolean",
                    "description": "Synthesise, place and route each block",
                    "default": False,
                },
                "as_if": {
                    "type": "string",
                    "description": "With synth: DEVICE[@MHZ], e.g. LFE5U-25F-6BG381C@25",
                },
                "machine": {
                    "type": "string",
                    "description": "With synth: only this machine (name or id)",
                },
                "keep_dir": KEEP_DIR,
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_soc",
        description=(
            "The processor and the programmable logic together (`arclang soc`): one LiteX "
            "SoC (a VexRiscv running the Zephyr application, the blocks of the machines "
            "allocated to programmable logic on its bus) simulated cycle by cycle in "
            "Verilator, each test case that `execute:`s such a machine run through the "
            "driver and compared with the fixed-point C and the model. `bitstream: true` (or "
            "`board`) also builds the same SoC into one bitstream; `gate_level: true` "
            "simulates the SoC as synthesised; `port` runs the cases on a board holding the "
            "bitstream, through its serial port, instead of simulating."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "test": TEST,
                "bitstream": {
                    "type": "boolean",
                    "description": "Build the bitstream, for the node's board",
                    "default": False,
                },
                "board": {
                    "type": "string",
                    "description": "Build the bitstream for this board (ulx3s, colorlight_i5, "
                    "colorlight_i9, ecp5_evn)",
                },
                "no_sim": {
                    "type": "boolean",
                    "description": "Do not simulate (with a bitstream: the bitstream only)",
                    "default": False,
                },
                "gate_level": {
                    "type": "boolean",
                    "description": "Simulate the SoC as synthesised for the ECP5",
                    "default": False,
                },
                "port": {
                    "type": "string",
                    "description": "Run the cases on the board through this serial port "
                    "(/dev/cu.usbserial..., /dev/ttyUSB0)",
                },
                "keep_dir": KEEP_DIR,
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_hil",
        description=(
            "Hardware in the loop (`arclang hil`): the generated C on a board, each test "
            "case that `execute:`s sent through the board's serial port with its input "
            "vectors, the outputs read back, compared with the model-level run step by step "
            "and its `expects:` judged on both. Needs the board: `port` is its serial port "
            "(opening it resets most boards). `command` is a program standing for the board "
            "(the harness built on this computer), accepted only when the server was started "
            "with ARCLANG_MCP_ALLOW_COMMANDS=1."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "port": {
                    "type": "string",
                    "description": "The board's serial port (/dev/cu.usbmodem..., /dev/ttyACM0)",
                },
                "command": {
                    "type": "string",
                    "description": "A program standing for the board. Needs "
                    "ARCLANG_MCP_ALLOW_COMMANDS=1",
                },
                "baud": {"type": "integer", "description": "The speed of the port (default 115200)"},
                "test": TEST,
                "row_timeout": {
                    "type": "integer",
                    "description": "Seconds to wait for the board to be ready, and for each row "
                    "it answers (`--timeout`, default 10)",
                },
                "as_json": AS_JSON,
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_twin",
        description=(
            "The digital twins of a model (`arclang twin`): each `twin`'s physical model "
            "driven by its measured data (`drives:`) in OpenModelica, compared to the "
            "measurements (`compares:`) and checked against its `expects:`. A run takes "
            "minutes. `list: true` lists the twins as JSON and runs nothing. `live: true` "
            "follows the twin's live data (mqtt, opcua, kafka, http) and runs it over the "
            "latest window at every refresh: it needs `for_seconds`, how long it follows "
            f"(at most {LIVE_SECONDS_MAX}), because a call that never ends answers nothing."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "twin": {"type": "string", "description": "Only this twin (id or name)"},
                "list": {
                    "type": "boolean",
                    "description": "Do not run: list the twins as JSON",
                    "default": False,
                },
                "as_json": {
                    "type": "boolean",
                    "description": "The result as JSON, with the curves; with live, one line per run",
                    "default": False,
                },
                "live": {
                    "type": "boolean",
                    "description": "Follow the twin's live data, for `for_seconds`",
                    "default": False,
                },
                "for_seconds": {
                    "type": "integer",
                    "description": f"With live (required): stop after this many seconds, at most {LIVE_SECONDS_MAX}",
                },
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_observe",
        description=(
            "Observe a board against the model (`arclang observe`): an instrument (a second "
            "board running MicroPython, its console on the serial port `instrument`) drives "
            "the inputs of the board (`drive`) and times its outputs (`watch`); what it "
            "played is replayed on the model at the moments the board's own loop ran, and "
            "the outputs are compared change by change. The answer is an agreement on what "
            "was played, with the real period of the board's loop: a MEASURE, never a proof "
            "(arclang_prove proves). `recorded_path` compares a recording kept earlier "
            "(`keep_path`) instead of playing a run: no hardware is needed then."
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "correspondence": {
                    "type": "string",
                    "description": "The correspondence observed (default: the only one of the model)",
                },
                "instrument": {
                    "type": "string",
                    "description": "The serial port of the instrument (/dev/cu.usbmodem...)",
                },
                "drive": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Inputs of the machine the instrument drives: NAME=PIN, or "
                    "NAME=PIN:1 when the pin rests high",
                },
                "watch": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Outputs of the machine the instrument watches: NAME=PIN",
                },
                "changes": {
                    "type": "integer",
                    "description": "How many times an input changes in the run (default 200)",
                },
                "seed": {
                    "type": "integer",
                    "description": "The seed of the run: the same seed plays the same inputs (default 1)",
                },
                "recorded_path": {
                    "type": "string",
                    "description": "Compare this recording, kept earlier, instead of playing a run",
                },
                "keep_path": {
                    "type": "string",
                    "description": "Keep the recording of the run in this file",
                },
                "as_json": {
                    "type": "boolean",
                    "description": "The comparison as JSON",
                    "default": False,
                },
                "timeout": RUN_TIMEOUT,
            },
            "model_path",
        ),
    ),
    Tool(
        name="arclang_debug",
        description=(
            "Run a behavior of the model step by step (`arclang debug`): a state machine "
            "(`machine`) with `events` fired in order (`-` is a completion step), a scenario "
            "(`scenario`) or a functional chain or operational process (`chain`) played to "
            "its end; `conditions` are the guards and conditions that hold, as written in "
            "the model. Each step says what is active, what fired and the lines of the "
            "model it ran. `list: true` lists what the model can run. Nothing is simulated "
            "outside the compiler and nothing is written. (The debugger protocol of VS Code, "
            "`--dap`, is a conversation on the standard input and is not offered.)"
        ),
        inputSchema=schema(
            {
                "model_path": MODEL_PATH,
                "machine": {"type": "string", "description": "The state machine to run, by name or id"},
                "scenario": {"type": "string", "description": "The scenario to play, by name or id"},
                "chain": {
                    "type": "string",
                    "description": "The functional chain (or operational process) to run, by name or id",
                },
                "events": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Events to fire in order (`-` for a completion step)",
                },
                "conditions": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Guards and conditions that hold, as written in the model (`--set`)",
                },
                "list": {
                    "type": "boolean",
                    "description": "List what the model can run, as JSON",
                    "default": False,
                },
                "as_json": {
                    "type": "boolean",
                    "description": "The steps as JSON",
                    "default": False,
                },
            },
            "model_path",
        ),
    ),
]


class SimulationTools:
    """Verify, the stages (cosim, sil, pil, hdl, soc, hil), twin, observe, debug."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_verify": self._verify,
            "arclang_cosim": self._cosim,
            "arclang_sil": self._sil,
            "arclang_pil": self._pil,
            "arclang_hdl": self._hdl,
            "arclang_soc": self._soc,
            "arclang_hil": self._hil,
            "arclang_twin": self._twin,
            "arclang_observe": self._observe,
            "arclang_debug": self._debug,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown simulation tool: {tool_name}")
        return await handler(arguments)

    def _model(self, args: Dict[str, Any]) -> Path:
        return resolve_path(self.workspace_root, args["model_path"])

    def _stage(self, args: Dict[str, Any], command: str) -> CommandLine:
        """What every stage takes: one case, the result as JSON."""
        line = CommandLine(self.workspace_root, args, command)
        return line.value("test", "--test").flag("as_json", "--json")

    async def _run(self, title: str, args: Dict[str, Any], arguments: List[str]) -> str:
        limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        return report(title, await self.compiler.run(arguments, timeout=limit))

    async def _verify(self, args: Dict[str, Any]) -> str:
        line = CommandLine(self.workspace_root, args, "verify")
        if args.get("check_tools", False):
            # What is installed: no model is read.
            run = await self.compiler.run(line.add("--check-tools").flag("as_json", "--json").parts)
            verdict = "what is installed (nothing was simulated)" if run.ok else None
            return report("Tools of the simulation", run, verdict)
        if not args.get("model_path"):
            raise InvalidArguments("'model_path' is required (only 'check_tools' needs no model)")
        model = self._model(args)
        modes = [
            key
            for key in ("list", "replay", "statistics", "drawings", "drawing")
            if args.get(key)
        ]
        if len(modes) > 1:
            raise InvalidArguments(f"{' and '.join(modes)} are different questions: ask one at a time")
        if args.get("point") is not None and "replay" not in modes:
            raise InvalidArguments("'point' goes with 'replay'")
        if "statistics" in modes and not args.get("test"):
            raise InvalidArguments("'statistics' analyzes one Monte Carlo case: name it with 'test'")
        if args.get("point") is not None and args["point"] < 0:
            raise InvalidArguments("'point' is 0 or more")
        line.value("test", "--test").flag("as_json", "--json")
        line.flag("list", "--list").value("replay", "--replay").value("point", "--point")
        line.value("statistics", "--statistics").flag("drawings", "--drawings")
        line.value("drawing", "--drawing")
        title = f"Verification by simulation of {model.name}"
        if modes and modes != ["statistics"] and modes != ["drawing"]:
            # Read from what is kept: nothing is simulated, and the first line says so.
            run = await self.compiler.run(line.end(model))
            verdict = f"{modes[0]} (nothing was simulated)" if run.ok else None
            return report(title, run, verdict)
        return await self._run(title, args, line.end(model))

    async def _cosim(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = self._stage(args, "cosim")
        if args.get("cc"):
            line.add(f"--cc={program_name(args['cc'], 'cc')}")
        line.path("keep_dir", "--keep")
        return await self._run(f"Co-simulation of {model.name}", args, line.end(model))

    async def _sil(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = self._stage(args, "sil")
        for compiler in args.get("cc") or []:
            line.add(f"--cc={program_name(compiler, 'cc')}")
        line.path("keep_dir", "--keep")
        return await self._run(f"Software in the loop of {model.name}", args, line.end(model))

    async def _pil(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = self._stage(args, "pil").value("target", "--target")
        line.path("keep_dir", "--keep")
        return await self._run(f"Processor in the loop of {model.name}", args, line.end(model))

    async def _hdl(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        if not args.get("synth", False) and (args.get("as_if") or args.get("machine")):
            raise InvalidArguments("'as_if' and 'machine' go with 'synth: true'")
        line = self._stage(args, "hdl").flag("synth", "--synth")
        line.value("as_if", "--as-if").value("machine", "--machine")
        line.path("keep_dir", "--keep")
        return await self._run(f"Hardware in simulation of {model.name}", args, line.end(model))

    async def _soc(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = self._stage(args, "soc").flag("bitstream", "--bitstream").value("board", "--board")
        line.flag("no_sim", "--no-sim").flag("gate_level", "--gate-level")
        port = serial_port(args, "port")
        if port:
            line.add(f"--port={port}")
        line.path("keep_dir", "--keep")
        return await self._run(f"Processor and logic together of {model.name}", args, line.end(model))

    async def _hil(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        port = serial_port(args, "port")
        stand_in = command_argument(
            args, "command", "Without it, give the serial port of the board ('port')."
        )
        if bool(port) == bool(stand_in):
            raise InvalidArguments("give the board: its serial 'port', or a 'command' standing for it")
        for key in ("baud", "row_timeout"):
            if args.get(key) is not None and args[key] < 1:
                raise InvalidArguments(f"'{key}' must be at least 1")
        line = self._stage(args, "hil").value("baud", "--baud").value("row_timeout", "--timeout")
        line.add(f"--port={port}" if port else f"--command={stand_in}")
        return await self._run(f"Hardware in the loop of {model.name}", args, line.end(model))

    async def _twin(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = CommandLine(self.workspace_root, args, "twin").value("twin", "--twin")
        line.flag("as_json", "--json")
        title = f"Digital twins of {model.name}"
        live = args.get("live", False)
        if args.get("list", False):
            if live:
                raise InvalidArguments("'list' runs nothing and 'live' follows a run: ask one")
            run = await self.compiler.run(line.add("--list").end(model))
            return report(title, run, "listed (nothing was run)" if run.ok else None)
        if not live:
            if args.get("for_seconds") is not None:
                raise InvalidArguments("'for_seconds' goes with 'live: true'")
            return await self._run(title, args, line.end(model))
        if args.get("for_seconds") is None:
            raise InvalidArguments(
                "'live' follows the data until stopped: say for how long with 'for_seconds' "
                f"(at most {LIVE_SECONDS_MAX})"
            )
        following = bounded_seconds(args, "for_seconds", 0, LIVE_SECONDS_MAX)
        line.add("--live", f"--for={following}")
        # The last window is still simulated when the time is up.
        limit = following + bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        run = await self.compiler.run(line.end(model), timeout=limit)
        return report(f"{title}, live for {following} s", run)

    async def _observe(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = CommandLine(self.workspace_root, args, "observe")
        line.value("correspondence", "--correspondence")
        instrument = serial_port(args, "instrument")
        recorded = line.path("recorded_path", "--recorded")
        if bool(instrument) == bool(recorded):
            raise InvalidArguments(
                "give what is compared: the serial port of the 'instrument' that plays a run, "
                "or a 'recorded_path' kept earlier"
            )
        for key in ("changes", "seed"):
            if args.get(key) is not None and args[key] < (1 if key == "changes" else 0):
                raise InvalidArguments(f"'{key}' is out of range")
        if instrument:
            line.add(f"--instrument={instrument}")
        line.each("drive", "--drive").each("watch", "--watch")
        line.value("changes", "--changes").value("seed", "--seed").flag("as_json", "--json")
        line.path("keep_path", "--keep", make_parent=True)
        limit = bounded_seconds(args, "timeout", RUN_SECONDS, RUN_SECONDS_MAX)
        run = await self.compiler.run(line.end(model), timeout=limit)
        # An agreement is a measure on what was played: the first line never calls it a proof.
        verdict: Optional[str] = None
        if run.ok:
            verdict = "measured, not proven (read the agreement below)"
        return report(f"Observation of the board against {model.name}", run, verdict)

    async def _debug(self, args: Dict[str, Any]) -> str:
        model = self._model(args)
        line = CommandLine(self.workspace_root, args, "debug")
        title = f"Step-by-step run of {model.name}"
        if args.get("list", False):
            run = await self.compiler.run(line.add("--list").end(model))
            return report(title, run, "what the model can run" if run.ok else None)
        targets = [key for key in ("machine", "scenario", "chain") if args.get(key)]
        if len(targets) > 1:
            raise InvalidArguments(f"{' and '.join(targets)}: one behavior is run at a time")
        line.value("machine", "--machine").value("scenario", "--scenario").value("chain", "--chain")
        line.joined("events", "--events").joined("conditions", "--set").flag("as_json", "--json")
        return report(title, await self.compiler.run(line.end(model)))
