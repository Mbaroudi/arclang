"""The engineering tool calls `arclang-server engineering`, which belongs to the closed edition.

That binary is not in this repository, so these tests run a STAND-IN script that prints what the
real one prints (its lines, `src/main.rs` of arclang-enterprise) and records the words it was given.
They prove what the tool does with those words: the command it builds, the refusals before anything
runs, and that the first line never says better than the output. They do not prove the engines.
"""

import json
import os
import stat
import sys
from pathlib import Path

import pytest

from arclang_mcp.tools.common import InvalidArguments
from arclang_mcp.tools.engineering import SERVER_VARIABLE, TOOLS

PASS = '''PN-FFT16 "16-point transform" (photonics, fft16.json)
  ✓ pass         relative_error < 2 %   measured 0.0087
1 verdict(s) kept in /tmp/x.verdicts.json'''
FAIL = '''PN-FFT16 "16-point transform" (photonics, fft16.json)
  ✗ FAIL         relative_error < 2 %   measured 0.0871'''
INCONCLUSIVE = '''PN-ARM "arm" (mechanics, arm.FCStd)
  ? inconclusive stress < 40 MPa   measured -'''
STALE = '''PN-FFT16 "16-point transform" (photonics, fft16.json)   STALE: what it was run on has changed, run it again
  ✓ pass         relative_error < 2 %   measured 0.0087'''
NOT_RUN = '''PN-FFT16 "16-point transform" (photonics, fft16.json)   NOT RUN'''

STAND_IN = f"""#!{sys.executable}
import json, os, sys
open(os.environ["STAND_IN_LOG"], "w").write(json.dumps(sys.argv[1:]))
sys.stdout.write(os.environ.get("STAND_IN_SAYS", ""))
sys.exit(int(os.environ.get("STAND_IN_EXIT", "0")))
"""


@pytest.fixture
def stand_in(workspace: Path, monkeypatch: pytest.MonkeyPatch):
    """A stand-in `arclang-server`: says what the test sets, records the words it was given."""
    program = workspace / "arclang-server-stand-in"
    program.write_text(STAND_IN)
    program.chmod(program.stat().st_mode | stat.S_IXUSR)
    log = workspace / "words.json"
    monkeypatch.setenv(SERVER_VARIABLE, str(program))
    monkeypatch.setenv("STAND_IN_LOG", str(log))

    def say(text: str, code: int = 0) -> None:
        monkeypatch.setenv("STAND_IN_SAYS", text)
        monkeypatch.setenv("STAND_IN_EXIT", str(code))

    def words() -> list:
        return json.loads(log.read_text())

    say.words = words  # type: ignore[attr-defined]
    return say


@pytest.fixture
def model(workspace: Path) -> Path:
    path = workspace / "fft16.arc"
    path.write_text('model M { name: "m" }\n')
    return path


def first_line(answer: str) -> str:
    return answer.splitlines()[0]


async def call(server, **arguments) -> str:
    return await server.call_tool("arclang_engineering", arguments)


def test_the_tool_is_listed_and_names_no_engine():
    assert [tool.name for tool in TOOLS] == ["arclang_engineering"]
    text = json.dumps(TOOLS[0].model_dump(), default=str).lower()
    for engine in ("asic", "photonic", "kicad", "freecad", "calculix", "openroad"):
        assert engine not in text, f"the public tool must not name '{engine}'"


async def test_without_the_enterprise_edition_nothing_runs_and_no_verdict_is_claimed(server, model, monkeypatch):
    monkeypatch.delenv(SERVER_VARIABLE, raising=False)
    monkeypatch.setenv("PATH", "/nonexistent")
    answer = await call(server, action="model", model_path=str(model))
    assert first_line(answer).endswith("UNAVAILABLE (the Enterprise edition is needed)"), answer
    assert "no verdict is known" in answer


async def test_a_server_that_cannot_start_is_a_failure_not_a_pass(server, model, monkeypatch):
    monkeypatch.setenv(SERVER_VARIABLE, "/nonexistent/arclang-server")
    answer = await call(server, action="status", model_path=str(model))
    assert "FAILED" in first_line(answer) and "cannot run" in answer


async def test_connectors_needs_no_model(server, stand_in):
    stand_in("photonics reads realized_by photonics:   tools: sax   image: x (present)\n")
    answer = await call(server, action="connectors")
    assert stand_in.words() == ["engineering", "connectors"]
    assert "present" in answer


@pytest.mark.parametrize(
    "action, says, code, expected",
    [
        ("model", PASS, 0, "every limit holds"),
        ("model", FAIL, 1, "A LIMIT DOES NOT HOLD"),
        ("model", INCONCLUSIVE, 1, "NOT JUDGED (a limit is inconclusive; this is not a pass)"),
        ("status", PASS, 0, "every verdict is fresh and passes"),
        ("status", STALE, 1, "STALE (what it was run on has changed; this is not a pass)"),
        ("status", NOT_RUN, 1, "NOT RUN (a part has no kept verdict; this is not a pass)"),
    ],
)
async def test_the_first_line_never_says_better_than_the_output(server, model, stand_in, action, says, code, expected):
    stand_in(says, code)
    answer = await call(server, action=action, model_path=str(model))
    assert first_line(answer).endswith(expected), answer
    assert says.splitlines()[0] in answer  # the server's own words come back as printed


async def test_a_pass_line_with_a_failing_exit_is_not_called_a_pass(server, model, stand_in):
    stand_in(PASS, 1)
    answer = await call(server, action="model", model_path=str(model))
    assert "FAILED (exit 1)" in first_line(answer), answer


async def test_each_action_builds_the_command_the_server_reads(server, model, stand_in):
    stand_in("ok\n")
    path = str(model)
    cases = [
        (dict(action="credibility", id="PN-FFT16"), ["engineering", "credibility", path, "PN-FFT16"]),
        (dict(action="history"), ["engineering", "history", path]),
        (dict(action="artifacts"), ["engineering", "artifacts", path]),
        (
            dict(action="impact", targets=["led.kicad_pcb", "MAT-ABS"], since="HEAD~2"),
            ["engineering", "impact", path, "led.kicad_pcb", "MAT-ABS", "--since", "HEAD~2"],
        ),
        (
            dict(action="size", part="PN-ARM", parameter="thickness", **{"from": 3, "to": 8, "step": 1}),
            ["engineering", "size", path, "--part", "PN-ARM", "--parameter", "thickness", "--from", "3", "--to", "8", "--step", "1"],
        ),
        (
            dict(action="trade", part="PN-MOUNT", materials=["MAT-PLA", "MAT-ALU"]),
            ["engineering", "trade", path, "--part", "PN-MOUNT", "--materials", "MAT-PLA,MAT-ALU"],
        ),
        (
            dict(action="propose-value", part="PN-LED", component="R1", series="E96", **{"from": 100, "to": 1000}),
            ["engineering", "propose-value", path, "--part", "PN-LED", "--component", "R1", "--series", "E96", "--from", "100", "--to", "1000"],
        ),
        (
            dict(action="propose-value", part="PN-LED", component="R1", **{"from": 47, "to": 1000}),
            ["engineering", "propose-value", path, "--part", "PN-LED", "--component", "R1", "--series", "E24", "--from", "47", "--to", "1000"],
        ),
        (
            dict(action="propose", part="PN-PLATE", volume=[0.7, 0.5, 0.4]),
            ["engineering", "propose", path, "--part", "PN-PLATE", "--volume", "0.7,0.5,0.4"],
        ),
        (
            dict(action="calibrate", twin="TW-LED", parameter="Vf", source="physics/LedBoard.mo", **{"from": 1.5, "to": 2.5}),
            ["engineering", "calibrate", path, "--twin", "TW-LED", "--parameter", "Vf", "--from", "1.5", "--to", "2.5", "--source", "physics/LedBoard.mo"],
        ),
    ]
    for arguments, expected in cases:
        await call(server, model_path=path, **arguments)
        assert stand_in.words() == expected, arguments


REFUSED = [
    dict(action="nothing"),
    dict(action="model", id="PN-1"),  # meaningless for model
    dict(action="credibility", id="--since"),  # an option is never a value
    dict(action="impact"),
    dict(action="impact", targets=["../secret"]),
    dict(action="impact", targets=["-rf"]),
    dict(action="impact", since="--output=x"),
    dict(action="size", part="PN-1", parameter="t", **{"from": 3, "to": 1, "step": 1}),
    dict(action="size", part="PN-1", parameter="t", **{"from": 0, "to": 100, "step": 1}),
    dict(action="size", part="PN-1", parameter="t", **{"from": 1, "to": 2, "step": 0}),
    dict(action="trade", part="PN-1", materials=["MAT-A"]),
    dict(action="trade", part="PN-1", materials=["MAT-A,MAT-B", "MAT-C"]),
    dict(action="propose-value", part="PN-1", component="R1", **{"from": 0, "to": 10}),
    dict(action="propose-value", part="PN-1", component="R1", **{"from": 1, "to": 5000}),
    dict(action="propose-value", part="PN-1", component="--out", **{"from": 1, "to": 10}),
    dict(action="propose-value", part="PN-1", component="R1", series="E24 --x", **{"from": 1, "to": 10}),
    dict(action="propose", part="PN-1", volume=[]),
    dict(action="propose", part="PN-1", volume=[0.05]),
    dict(action="propose", part="PN-1", volume=[0.95]),
    dict(action="propose", part="PN-1", volume=["0.5"]),
    dict(action="propose", part="PN-1", volume=[0.5], out="proposals"),  # the tool writes nothing
    dict(action="calibrate", twin="TW-1", parameter="Vf", **{"from": 3, "to": 1}),
    dict(action="calibrate", twin="TW-1", parameter="Vf", source="../secret", **{"from": 1, "to": 3}),
    dict(action="calibrate", twin="--twin", parameter="Vf", **{"from": 1, "to": 3}),
    dict(action="model", timeout=0),
    dict(action="model", timeout=99999),
]


@pytest.mark.parametrize("arguments", REFUSED)
async def test_what_is_refused_is_refused_before_anything_runs(server, model, stand_in, arguments):
    stand_in("must not run\n")
    with pytest.raises(InvalidArguments):
        await _direct(server, {**arguments, "model_path": str(model)})
    assert not Path(os.environ["STAND_IN_LOG"]).exists(), "the server must not have been run"


@pytest.mark.parametrize(
    "arguments",
    [dict(action="model"), dict(action="connectors", model_path="x.arc")],
)
async def test_a_model_is_given_exactly_where_it_is_needed(server, stand_in, arguments):
    stand_in("must not run\n")
    with pytest.raises(InvalidArguments):
        await _direct(server, arguments)


async def _direct(server, arguments):
    return await server.engineering_tools.execute("arclang_engineering", arguments)


async def test_a_model_outside_the_workspace_is_refused(server, stand_in, tmp_path_factory):
    stand_in("must not run\n")
    outside = tmp_path_factory.mktemp("elsewhere") / "m.arc"
    outside.write_text("x")
    with pytest.raises(InvalidArguments):
        await _direct(server, {"action": "model", "model_path": str(outside)})


async def test_the_environment_is_not_printed(server, model, stand_in, monkeypatch):
    monkeypatch.setenv("ARCLANG_ENGINE_TOKEN", "secret-token-value-that-must-not-appear-0000")
    stand_in(PASS, 0)
    answer = await call(server, action="model", model_path=str(model))
    assert "secret-token" not in answer
