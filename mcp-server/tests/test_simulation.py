"""Verify, the stages, the twins, a board observed, a behavior stepped.

What simulates, builds for a target or drives a board is not run here: its
command line is recorded and read (`recorded`). What needs no such tool is run
with the real binary: the lists, a recorded observation, the step-by-step run,
and every refusal.
"""

import json
import shutil
from pathlib import Path

import pytest

from arclang_mcp.compiler.wrapper import Run
from arclang_mcp.tools.core import RUN_SECONDS

FIXTURES = Path(__file__).resolve().parents[2] / "tests" / "fixtures"
STAGES = ("arclang_cosim", "arclang_sil", "arclang_pil", "arclang_hdl", "arclang_soc")


# ---------------------------------------------------------------------- verify


async def test_verify_lists_the_cases_that_simulate_and_simulates_none(server, model):
    answer = await server.call_tool("arclang_verify", {"model_path": str(model), "list": True})
    assert answer.splitlines()[0] == (
        "Verification by simulation of braking.arc: list (nothing was simulated)"
    ), answer
    # The reference model has no case that simulates: the compiler's list is empty.
    assert json.loads(answer.split("\n\n", 1)[1]) == []


async def test_verify_says_what_is_installed_without_a_model(server):
    answer = await server.call_tool("arclang_verify", {"check_tools": True})
    first = answer.splitlines()[0]
    assert first.startswith("Tools of the simulation: "), answer
    assert "OpenModelica" in answer or "omc" in answer


async def test_verify_writes_the_run_and_gives_it_time(server, recorded):
    await server.call_tool("arclang_verify", {"model_path": "braking.arc", "test": "TC-001", "as_json": True})
    assert recorded.last[:3] == ["verify", "--test=TC-001", "--json"] and recorded.last[-2] == "--"
    assert recorded.timeouts[-1] == RUN_SECONDS
    await server.call_tool(
        "arclang_verify", {"model_path": "braking.arc", "test": "TC-MC", "statistics": "python", "timeout": 60}
    )
    assert "--statistics=python" in recorded.last and recorded.timeouts[-1] == 60
    await server.call_tool("arclang_verify", {"model_path": "braking.arc", "replay": "TC-001", "point": 2})
    assert {"--replay=TC-001", "--point=2"} <= set(recorded.last)


async def test_a_verification_that_fails_is_a_failure(server, recorded):
    recorded.reply = Run((), 1, "✗ TC-001: final(vehicle.distance) >= 300 km: 212 km\n", "")
    answer = await server.call_tool("arclang_verify", {"model_path": "braking.arc"})
    assert answer.splitlines()[0] == "Verification by simulation of braking.arc: FAILED (exit 1)"
    assert "212 km" in answer


@pytest.mark.parametrize(
    "arguments, mention",
    [
        ({}, "'model_path' is required"),
        ({"model_path": "braking.arc", "list": True, "replay": "TC-001"}, "one at a time"),
        ({"model_path": "braking.arc", "point": 1}, "'point' goes with 'replay'"),
        ({"model_path": "braking.arc", "statistics": "python"}, "name it with 'test'"),
        ({"model_path": "braking.arc", "statistics": "matlab", "test": "T"}, "must be one of"),
        ({"model_path": "braking.arc", "timeout": 0}, "'timeout'"),
        ({"model_path": "braking.arc", "timeout": 10**6}, "'timeout'"),
    ],
)
async def test_verify_refuses_what_it_cannot_mean(server, recorded, arguments, mention):
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_verify", arguments)
    assert mention in str(refused.value)
    assert recorded.commands == []


# ------------------------------------------------------------------ the stages


async def test_each_stage_runs_its_own_command_on_one_case(server, recorded):
    for tool in STAGES:
        await server.call_tool(tool, {"model_path": "braking.arc", "test": "TC-001", "as_json": True, "keep_dir": "kept"})
        command = recorded.last
        assert command[0] == tool.removeprefix("arclang_"), command
        assert {"--test=TC-001", "--json"} <= set(command)
        assert any(a.startswith("--keep=") and a.endswith("/kept") for a in command), command
        assert command[-2] == "--" and command[-1].endswith("braking.arc")
        assert recorded.timeouts[-1] == RUN_SECONDS


async def test_a_stage_reports_the_compilers_exit_and_words(server, recorded):
    recorded.reply = Run((), 1, "", "Error: no C compiler: `cc` is not on the PATH")
    for tool in STAGES:
        answer = await server.call_tool(tool, {"model_path": "braking.arc"})
        assert answer.splitlines()[0].endswith(": FAILED (exit 1)"), answer
        assert "no C compiler" in answer


async def test_sil_with_no_case_that_executes_says_so(server, model):
    """The real compiler, on a model with nothing to run as C: no compiler is launched."""
    answer = await server.call_tool("arclang_sil", {"model_path": str(model)})
    assert answer.startswith("Software in the loop of braking.arc: "), answer
    assert "execute" in answer


async def test_the_c_compilers_are_names_on_the_path(server, recorded):
    await server.call_tool("arclang_sil", {"model_path": "braking.arc", "cc": ["gcc", "clang", "gcc-15"]})
    assert [a for a in recorded.last if a.startswith("--cc")] == ["--cc=gcc", "--cc=clang", "--cc=gcc-15"]
    await server.call_tool("arclang_cosim", {"model_path": "braking.arc", "cc": "clang"})
    assert "--cc=clang" in recorded.last
    for bad in ("./evil.sh", "/bin/sh", "gcc -o /tmp/x", "--help", "$(id)", ""):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_sil", {"model_path": "braking.arc", "cc": [bad]})
        assert "bare name of a program" in str(refused.value), bad
    with pytest.raises(Exception):
        await server.call_tool("arclang_cosim", {"model_path": "braking.arc", "cc": "./evil.sh"})


async def test_hdl_and_soc_write_their_own_options(server, recorded):
    await server.call_tool(
        "arclang_hdl",
        {"model_path": "braking.arc", "synth": True, "as_if": "LFE5U-25F-6BG381C@25", "machine": "Wiper"},
    )
    assert {"--synth", "--as-if=LFE5U-25F-6BG381C@25", "--machine=Wiper"} <= set(recorded.last)
    with pytest.raises(Exception, match="go with 'synth: true'"):
        await server.call_tool("arclang_hdl", {"model_path": "braking.arc", "as_if": "LFE5U-25F"})
    await server.call_tool(
        "arclang_soc",
        {"model_path": "braking.arc", "board": "ulx3s", "no_sim": True, "gate_level": True, "bitstream": True},
    )
    assert {"--board=ulx3s", "--no-sim", "--gate-level", "--bitstream"} <= set(recorded.last)
    await server.call_tool("arclang_soc", {"model_path": "braking.arc", "port": "/dev/ttyUSB0"})
    assert "--port=/dev/ttyUSB0" in recorded.last
    await server.call_tool("arclang_pil", {"model_path": "braking.arc", "target": "zephyr/litex_vexriscv"})
    assert "--target=zephyr/litex_vexriscv" in recorded.last


async def test_a_serial_port_is_a_device_name(server, recorded):
    for bad in ("../../etc/passwd", "/etc/passwd", "/dev/../etc/passwd", "ttyACM0; rm -rf ~", "--help"):
        for tool, key in (("arclang_soc", "port"), ("arclang_hil", "port"), ("arclang_observe", "instrument")):
            with pytest.raises(Exception) as refused:
                await server.call_tool(tool, {"model_path": "braking.arc", key: bad})
            assert "serial port" in str(refused.value), (tool, bad)
    assert recorded.commands == []


async def test_hil_needs_a_board_and_runs_no_program_a_call_names(server, recorded):
    with pytest.raises(Exception, match="give the board"):
        await server.call_tool("arclang_hil", {"model_path": "braking.arc"})
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_hil", {"model_path": "braking.arc", "command": "./harness"})
    assert "ARCLANG_MCP_ALLOW_COMMANDS=1" in str(refused.value)
    assert recorded.commands == []
    await server.call_tool(
        "arclang_hil",
        {"model_path": "braking.arc", "port": "/dev/cu.usbmodem1101", "baud": 9600, "row_timeout": 3, "test": "TC-1"},
    )
    assert {"--port=/dev/cu.usbmodem1101", "--baud=9600", "--timeout=3", "--test=TC-1"} <= set(recorded.last)


async def test_hil_takes_a_stand_in_when_the_server_was_allowed_to(server, recorded, allow_commands):
    await server.call_tool("arclang_hil", {"model_path": "braking.arc", "command": "./build/harness"})
    assert "--command=./build/harness" in recorded.last
    with pytest.raises(Exception, match="give the board"):
        await server.call_tool(
            "arclang_hil", {"model_path": "braking.arc", "command": "./h", "port": "/dev/ttyACM0"}
        )


# ----------------------------------------------------------------------- twins


async def test_twin_lists_the_twins_and_runs_none(server, model):
    answer = await server.call_tool("arclang_twin", {"model_path": str(model), "list": True})
    assert answer.splitlines()[0] == "Digital twins of braking.arc: listed (nothing was run)", answer
    assert json.loads(answer.split("\n\n", 1)[1]) == []


async def test_twin_follows_live_data_only_for_a_time_it_is_given(server, recorded):
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_twin", {"model_path": "braking.arc", "live": True})
    assert "for_seconds" in str(refused.value)
    for too_long in (0, 3601, -5):
        with pytest.raises(Exception):
            await server.call_tool("arclang_twin", {"model_path": "braking.arc", "live": True, "for_seconds": too_long})
    with pytest.raises(Exception, match="goes with 'live: true'"):
        await server.call_tool("arclang_twin", {"model_path": "braking.arc", "for_seconds": 5})
    assert recorded.commands == []
    answer = await server.call_tool(
        "arclang_twin", {"model_path": "braking.arc", "twin": "TW-1", "live": True, "for_seconds": 30, "as_json": True}
    )
    assert {"--twin=TW-1", "--live", "--for=30", "--json"} <= set(recorded.last)
    # The server waits for the time it asked, and for the last window after it.
    assert recorded.timeouts[-1] == 30 + RUN_SECONDS
    assert "live for 30 s" in answer.splitlines()[0]


async def test_twin_runs_the_twins_without_live_unless_asked(server, recorded):
    await server.call_tool("arclang_twin", {"model_path": "braking.arc", "twin": "TW-JANUARY"})
    assert recorded.last[:2] == ["twin", "--twin=TW-JANUARY"]
    assert "--live" not in recorded.last and "--list" not in recorded.last


# --------------------------------------------------------------------- observe


@pytest.fixture
def lamp(workspace):
    """A model, its sketch, and a run recorded on a real Arduino UNO by a second board."""
    for name in ("uno_lamp.arc", "uno_lamp.ino"):
        shutil.copy(FIXTURES / "sketches" / name, workspace / name)
    shutil.copy(FIXTURES / "observations" / "uno_lamp_400.json", workspace / "run.json")
    return workspace / "uno_lamp.arc"


async def test_a_recorded_run_is_compared_and_called_a_measure(server, lamp):
    answer = await server.call_tool("arclang_observe", {"model_path": "uno_lamp.arc", "recorded_path": "run.json"})
    first = answer.splitlines()[0]
    assert first == "Observation of the board against uno_lamp.arc: measured, not proven (read the agreement below)"
    assert "✓ Agrees on this run" in answer and "the board's loop takes 10.024 ms" in answer
    assert "not a proof" in answer


async def test_a_model_the_board_does_not_follow_is_a_failure(server, lamp):
    lamp.write_text(lamp.read_text().replace('action: "led := not led"', 'action: "led := true"'))
    answer = await server.call_tool("arclang_observe", {"model_path": "uno_lamp.arc", "recorded_path": "run.json"})
    first = answer.splitlines()[0]
    assert "FAILED" in first and "measured" not in first, answer


async def test_observe_plays_a_run_through_the_instrument(server, recorded):
    with pytest.raises(Exception, match="give what is compared"):
        await server.call_tool("arclang_observe", {"model_path": "braking.arc"})
    await server.call_tool(
        "arclang_observe",
        {
            "model_path": "braking.arc",
            "instrument": "/dev/cu.usbmodem1101",
            "drive": ["button=2:1"],
            "watch": ["led=13"],
            "changes": 400,
            "seed": 7,
            "keep_path": "runs/run.json",
            "correspondence": "Uno lamp sketch",
        },
    )
    assert {
        "--instrument=/dev/cu.usbmodem1101", "--drive=button=2:1", "--watch=led=13",
        "--changes=400", "--seed=7", "--correspondence=Uno lamp sketch",
    } <= set(recorded.last)
    assert any(a.startswith("--keep=") and a.endswith("runs/run.json") for a in recorded.last)


# ----------------------------------------------------------------------- debug


async def test_debug_lists_what_the_model_can_run(server, model):
    answer = await server.call_tool("arclang_debug", {"model_path": str(model), "list": True})
    assert answer.splitlines()[0] == "Step-by-step run of braking.arc: what the model can run"
    listed = json.loads(answer.split("\n\n", 1)[1])
    assert listed == {"chains": [], "machines": ["BrakingModes"], "scenarios": ["EmergencyStop"]}


async def test_debug_fires_the_events_in_order(server, model):
    answer = await server.call_tool(
        "arclang_debug", {"model_path": str(model), "machine": "BrakingModes", "events": ["LF-001", "LF-002"]}
    )
    assert answer.splitlines()[0] == "Step-by-step run of braking.arc: succeeded", answer
    assert answer.index("active: Braking") < answer.rindex("active: Standby")
    as_json = await server.call_tool(
        "arclang_debug",
        {"model_path": str(model), "machine": "BrakingModes", "events": ["SF-001"], "conditions": ["sensor_fault"], "as_json": True},
    )
    steps = json.loads(as_json.split("\n\n", 1)[1])
    assert steps["kind"] == "state machine" and "Failed" in steps["steps"][-1]["text"]


async def test_debug_plays_a_scenario(server, model):
    answer = await server.call_tool("arclang_debug", {"model_path": str(model), "scenario": "EmergencyStop"})
    assert "critical threat" in answer


async def test_debug_runs_one_behavior_and_never_the_debugger_protocol(server, recorded):
    with pytest.raises(Exception, match="one behavior is run at a time"):
        await server.call_tool("arclang_debug", {"model_path": "braking.arc", "machine": "M", "scenario": "S"})
    with pytest.raises(Exception, match="'dap' is not an argument"):
        await server.call_tool("arclang_debug", {"model_path": "braking.arc", "dap": True})
    with pytest.raises(Exception, match="must not hold a comma"):
        await server.call_tool("arclang_debug", {"model_path": "braking.arc", "events": ["a,b"]})
    assert recorded.commands == []
    await server.call_tool("arclang_debug", {"model_path": "braking.arc", "chain": "FC-1", "events": ["a", "-", "b"]})
    assert {"--chain=FC-1", "--events=a,-,b"} <= set(recorded.last) and "--dap" not in recorded.last


# ------------------------------------------------- realize, partition, schedule


async def test_realize_climbs_only_when_told_to_run(server, recorded):
    await server.call_tool("arclang_realize", {"model_path": "braking.arc", "requirement": "REQ-1", "up_to": "sil"})
    assert {"--dry-run", "--json", "--requirement=REQ-1", "--up-to=sil"} <= set(recorded.last)
    answer = await server.call_tool(
        "arclang_realize", {"model_path": "braking.arc", "run": True, "keep_going": True, "board": True, "timeout": 1200}
    )
    assert "--dry-run" not in recorded.last
    assert {"--keep-going", "--board", "--json"} <= set(recorded.last)
    assert recorded.timeouts[-1] == 1200
    assert answer.startswith("Climb of the realization ladder of braking.arc: succeeded")
    with pytest.raises(Exception, match="goes with 'run: true'"):
        await server.call_tool("arclang_realize", {"model_path": "braking.arc", "keep_going": True})
    for not_a_yes in ("true", 1, "yes"):
        with pytest.raises(Exception, match="'run' must be a boolean"):
            await server.call_tool("arclang_realize", {"model_path": "braking.arc", "run": not_a_yes})


async def test_a_climb_that_fails_is_a_failure(server, recorded):
    recorded.reply = Run((), 1, '{"runs":[{"stage":"sil","status":"fail"}]}', "")
    answer = await server.call_tool("arclang_realize", {"model_path": "braking.arc", "run": True})
    assert answer.splitlines()[0] == "Climb of the realization ladder of braking.arc: FAILED (exit 1)"


async def test_partition_measures_only_when_told_to_run(server, recorded):
    await server.call_tool("arclang_partition", {"model_path": "braking.arc", "clock_mhz": 50})
    assert recorded.commands[0][:1] == ["partition"] and "--dry-run" in recorded.commands[0]
    assert "--clock=50" in recorded.commands[0]
    recorded.commands.clear()
    answer = await server.call_tool(
        "arclang_partition",
        {"model_path": "braking.arc", "run": True, "as_json": True, "machine": "Wiper", "device": "LFE5U-25F-6BG381C"},
    )
    assert len(recorded.commands) == 1 and "--dry-run" not in recorded.last
    assert {"--json", "--machine=Wiper", "--device=LFE5U-25F-6BG381C"} <= set(recorded.last)
    assert recorded.timeouts[-1] == RUN_SECONDS
    assert "(measured)" in answer.splitlines()[0]
    with pytest.raises(Exception, match="goes with 'run: true'"):
        await server.call_tool("arclang_partition", {"model_path": "braking.arc", "as_json": True})


async def test_schedule_proves_on_the_emulator_only_when_asked(server, recorded):
    await server.call_tool("arclang_schedule", {"model_path": "braking.arc", "wcet": {"Wiper": 120}})
    assert "--prove" not in recorded.last and "--wcet=Wiper=120" in recorded.last
    assert recorded.timeouts[-1] is None
    await server.call_tool(
        "arclang_schedule", {"model_path": "braking.arc", "prove": True, "cycles": 4, "keep_dir": "kept"}
    )
    assert {"--prove", "--cycles=4", "--json"} <= set(recorded.last)
    assert recorded.timeouts[-1] == RUN_SECONDS
    with pytest.raises(Exception, match="go with 'prove: true'"):
        await server.call_tool("arclang_schedule", {"model_path": "braking.arc", "cycles": 4})
