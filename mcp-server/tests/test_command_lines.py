"""Every option a tool is said to reach is really written on the command line.

`tools/cli.py` says how each option of the CLI is reached: by an argument of
the same name, or by another one (`Via`). Saying so is not doing so. Here each
of those arguments is given to its tool, and the command line the server
builds must carry the option. The compiler is NOT run for this: the runs are
recorded instead, because many of these commands simulate, build for a target
or drive a board (OpenModelica, cross compilers, GHDL, Renode, a serial port),
and a test must launch none of them. What the commands answer is tested with
the real binary in the other files, through their cheap paths.
"""

from typing import Any, Dict, List, Tuple

from arclang_mcp.tools import cli
from arclang_mcp.tools.common import input_schema

from test_cli_parity import help_of, options_of

EDIT = {"op": "remove", "element": "LC-001"}

# The arguments a tool needs before any other is looked at.
BASE: Dict[str, Dict[str, Any]] = {
    "arclang_diff": {},
    "arclang_merge": {"base_path": "braking.arc", "ours_path": "braking.arc", "theirs_path": "braking.arc"},
    "arclang_git_merge": {"base_path": "braking.arc", "ours_path": "braking.arc", "theirs_path": "braking.arc"},
    "arclang_history": {"model_path": "braking.arc", "element": "LC-001"},
    "arclang_restore": {"model_path": "braking.arc", "rev": "HEAD"},
    "arclang_new": {"name": "door"},
    "arclang_report": {"from_rev": "main"},
    "arclang_sarif": {},
    "arclang_ci_init": {"directory": "."},
    "arclang_ci_run": {},
    "arclang_ci_models": {},
    "arclang_rules": {},
    "arclang_metamodel": {},
    "arclang_set_attribute": {"model_path": "braking.arc", "element": "LC-001", "key": "k", "value": "1"},
    "arclang_unset_attribute": {"model_path": "braking.arc", "element": "LC-001", "key": "k"},
    "arclang_rename": {"model_path": "braking.arc", "element": "LC-001", "name": "N"},
    "arclang_workbench": {"path": "braking.arc"},
    "arclang_edit": {"path": "braking.arc", "requests": [EDIT]},
    "arclang_format": {"path": "braking.arc"},
    "arclang_import": {"input_path": "in.json", "format": "json", "output_path": "out.arc"},
    "arclang_recover": {"code_path": "c"},
    "arclang_drift": {"model_path": "braking.arc", "code_path": "c"},
    "arclang_safety_check": {"model_path": "braking.arc", "standard": "iso26262"},
    "arclang_export": {"model_path": "braking.arc", "format": "json"},
    "arclang_document": {"model_path": "braking.arc", "output_path": "review.md"},
    "arclang_generate_diagram": {"model_path": "braking.arc", "diagram_type": "mermaid"},
    "arclang_observe": {"model_path": "braking.arc", "recorded_path": "run.json"},
    "arclang_baseline": {},
    "arclang_transition": {"model_path": "braking.arc", "step": "sa-to-la"},
}

# For a tool that runs several subcommands: what chooses the one looked at.
CHOICE: Dict[str, Dict[str, Any]] = {
    "baseline create": {"action": "create", "name": "v1"},
    "baseline list": {"action": "list"},
    "baseline diff": {"action": "diff", "a": "v1", "b": "v2"},
    "baseline verify": {"action": "verify", "name": "v1"},
}
# The short spelling the first tools write.
SHORT = {"--format": "-f", "--output": "-o"}

# What else an argument needs to mean something: (tool, argument) -> the other arguments.
CONTEXT: Dict[Tuple[str, str], Dict[str, Any]] = {
    ("arclang_diff", "old_path"): {"new_path": "braking.arc"},
    ("arclang_diff", "new_path"): {"old_path": "braking.arc"},
    ("arclang_diff", "model_path"): {"from_rev": "HEAD"},
    ("arclang_diff", "from_rev"): {"model_path": "braking.arc"},
    ("arclang_diff", "to_rev"): {"model_path": "braking.arc", "from_rev": "HEAD~1"},
    ("arclang_diff", "format"): {"old_path": "braking.arc", "new_path": "braking.arc"},
    ("arclang_diff", "output_path"): {"old_path": "braking.arc", "new_path": "braking.arc"},
    ("arclang_diff", "gate_standard"): {"model_path": "braking.arc", "from_rev": "HEAD~1"},
    ("arclang_trace_analysis", "from"): {"review": True},
    ("arclang_trace_analysis", "to"): {"review": True},
    ("arclang_impact", "current_path"): {"since": "HEAD"},
    ("arclang_impact", "format"): {"element": "LC-001", "output_path": "impact.xlsx"},
    ("arclang_impact", "depth"): {"element": "LC-001"},
    ("arclang_impact", "kinds"): {"element": "LC-001"},
    ("arclang_impact", "direction"): {"element": "LC-001"},
    ("arclang_impact", "gate_standard"): {"element": "LC-001"},
    ("arclang_impact", "output_path"): {"element": "LC-001"},
    ("arclang_realize", "keep_going"): {"run": True},
    ("arclang_partition", "as_json"): {"run": True},
    ("arclang_schedule", "cycles"): {"prove": True},
    ("arclang_schedule", "keep_dir"): {"prove": True},
    ("arclang_viewpoint_diagrams", "theme"): {"format": "html"},
    ("arclang_recover", "with"): {"propose": True, "code_path": "toggle.ino"},
    ("arclang_recover", "turns"): {"propose": True, "code_path": "toggle.ino"},
    ("arclang_recover", "timeout"): {"propose": True, "code_path": "toggle.ino"},
    ("arclang_recover", "keep_dir"): {"code_path": "firmware.hex"},
    ("arclang_recover", "model_path"): {"code_path": "firmware.hex"},
    ("arclang_drift", "timeout"): {"prove": True},
    ("arclang_verify", "point"): {"model_path": "braking.arc", "replay": "TC-001"},
    ("arclang_verify", "statistics"): {"model_path": "braking.arc", "test": "TC-001"},
    ("arclang_hdl", "as_if"): {"synth": True},
    ("arclang_hdl", "machine"): {"synth": True},
    ("arclang_hil", "baud"): {"port": "/dev/ttyACM0"},
    ("arclang_hil", "test"): {"port": "/dev/ttyACM0"},
    ("arclang_hil", "row_timeout"): {"port": "/dev/ttyACM0"},
    ("arclang_hil", "as_json"): {"port": "/dev/ttyACM0"},
    ("arclang_twin", "for_seconds"): {"live": True},
    ("arclang_twin", "live"): {"for_seconds": 5},
    ("arclang_observe", "instrument"): {"recorded_path": None},
    ("arclang_transition", "plan"): {"step": None},
    ("arclang_table", "format"): {"output_path": "table.xlsx"},
    ("arclang_ci_init", "force"): {"write": True},
}

# The value given to an argument when its type alone does not say a sensible one.
VALUES: Dict[str, Any] = {
    "port": "/dev/ttyACM0",
    "instrument": "/dev/ttyACM0",
    "theme": "print-grayscale",
    "change": EDIT,
    "wcet": {"Wiper": 120.0},
    "compile": False,
    "requests": [EDIT],
    "output_path": "out/result.md",
    "models": ["braking.arc"],
}
VALUES_OF: Dict[Tuple[str, str], Any] = {
    ("arclang_cosim", "cc"): "gcc",
    ("arclang_sil", "cc"): ["gcc", "clang"],
    ("arclang_ci_run", "verify"): "auto",
}


def sample(tool: str, argument: str, declared: Dict[str, Any]) -> Any:
    if (tool, argument) in VALUES_OF:
        return VALUES_OF[(tool, argument)]
    if argument in VALUES:
        return VALUES[argument]
    kind = declared.get("type")
    if "enum" in declared:
        return declared["enum"][-1]
    if kind == "boolean":
        return True
    if kind == "integer":
        return 3
    if kind == "number":
        return 3.0
    if kind == "array":
        items = declared.get("items", {})
        return [items["enum"][0]] if "enum" in items else ["x"]
    return "x"


def carries(commands: List[List[str]], subcommand: str, option: str) -> bool:
    words = subcommand.split()
    return any(
        command[: len(words)] == words
        and any(a in (option, SHORT.get(option)) or a.startswith(option + "=") for a in command)
        for command in commands
    )


async def test_every_argument_said_to_reach_an_option_writes_it(server, recorded, allow_commands):
    # The help is read from the real binary, on a compiler of its own: only the tools' runs are recorded.
    from arclang_mcp.compiler.wrapper import ArcLangCompiler

    class Real:
        compiler = ArcLangCompiler({})

    problems = []
    checked = 0
    for subcommand in cli.covered_subcommands():
        printed = set(options_of(await help_of(Real, *subcommand.split()))) - set(cli.GLOBAL_OPTIONS)
        for option in sorted(printed):
            reach = cli.OPTIONS[subcommand].get(option)
            if isinstance(reach, (cli.ByServer, cli.Never)):
                continue
            argument = reach.argument if isinstance(reach, cli.Via) else option[2:].replace("-", "_")
            tools = [
                tool
                for tool in cli.tools_running(subcommand)
                if argument in input_schema(server.tools_by_name[tool])["properties"]
            ]
            assert tools, (subcommand, option)
            written = False
            said = ""
            for tool in tools:
                declared = input_schema(server.tools_by_name[tool])["properties"][argument]
                call = {
                    **BASE.get(tool, {"model_path": "braking.arc"}),
                    **CHOICE.get(subcommand, {}),
                    **CONTEXT.get((tool, argument), {}),
                }
                call = {key: value for key, value in call.items() if value is not None}
                call[argument] = sample(tool, argument, declared)
                recorded.commands.clear()
                try:
                    await server.call_tool(tool, call)
                except Exception as refused:  # the call was not built: say why below
                    said = f"{tool}({call}) was refused: {refused}"
                    continue
                if carries(recorded.commands, subcommand, option):
                    written = True
                    break
                said = f"{tool}({call}) ran {recorded.commands}"
            checked += 1
            if not written:
                problems.append(f"arclang {subcommand} {option} is not written: {said}")
    assert not problems, "\n".join(problems)
    assert checked > 150, checked


async def test_no_tool_writes_an_option_said_never_to_be_offered(server, recorded, allow_commands):
    """Every argument of every tool, given at once where the tool allows: `--open`, `--dap`, `--install` never appear."""
    never = {
        option
        for declared in cli.OPTIONS.values()
        for option, reach in declared.items()
        if isinstance(reach, cli.Never)
    } - {"--dry-run", "--yes", "--binary", "--dependencies"}
    assert {"--open", "--dap", "--install"} <= never
    for tool in cli.COVERS:
        declared = input_schema(server.tools_by_name[tool])["properties"]
        for argument, said in declared.items():
            for choice in ([CHOICE[c] for c in cli.COVERS[tool] if c in CHOICE] or [{}]):
                call = {
                    **BASE.get(tool, {"model_path": "braking.arc"}),
                    **choice,
                    **CONTEXT.get((tool, argument), {}),
                }
                call = {key: value for key, value in call.items() if value is not None}
                call[argument] = sample(tool, argument, said)
                try:
                    await server.call_tool(tool, call)
                except Exception:
                    continue
    written = {a.split("=")[0] for command in recorded.commands for a in command if a.startswith("--")}
    assert len(recorded.commands) > 300
    assert not written & never, written & never
