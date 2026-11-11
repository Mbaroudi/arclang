"""Prove, recover and drift are the compiler's: the answer never says more than it printed."""

import shutil
from pathlib import Path

import pytest

SKETCHES = Path(__file__).resolve().parents[2] / "tests" / "fixtures" / "sketches"
HAS_PROVERS = bool(shutil.which("cbmc") and shutil.which("bitwuzla"))


@pytest.fixture
def sketch(workspace):
    """A sketch written by hand and the model that says what it is, in the workspace."""
    for name in ("toggle.ino", "toggle.arc"):
        shutil.copy(SKETCHES / name, workspace / name)
    return workspace / "toggle.arc"


async def generated_c(server, model, workspace, name="c"):
    run = await server.compiler.run(["export", str(model), "-f", "c", "-o", str(workspace / name)])
    assert run.ok, run.output
    return workspace / name


async def test_the_three_tools_are_listed(server):
    names = {tool.name for tool in server.tools}
    assert {"arclang_prove", "arclang_recover", "arclang_drift"} <= names


async def test_prove_relays_the_proof_or_says_the_provers_are_missing(server, sketch):
    answer = await server.call_tool("arclang_prove", {"model_path": str(sketch)})
    first = answer.splitlines()[0]
    if HAS_PROVERS:
        assert first == "Proof of the correspondences of toggle.arc: proven", answer
        assert 'toggle.ino is state_machine "Lamp"' in answer
    else:
        # No prover here: a failure that says where they are, never a proof.
        assert "FAILED" in first and "proven" not in first, answer
        assert "tools/embedded/formal" in answer


@pytest.mark.skipif(not HAS_PROVERS, reason="needs CBMC and Bitwuzla (ArcLang's formal image)")
async def test_a_refuted_proof_is_told_with_the_lines_and_the_case(server, sketch):
    sketch.write_text(sketch.read_text().replace('guard: "button" }', 'guard: "not button" }', 1))
    answer = await server.call_tool("arclang_prove", {"model_path": str(sketch), "timeout": 600})
    assert answer.splitlines()[0] == "Proof of the correspondences of toggle.arc: REFUTED", answer
    assert "does not hold after it:" in answer and "a case that shows it" in answer


async def test_prove_refuses_a_time_it_cannot_keep(server, sketch):
    for timeout in (0, 100000, "soon", True):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_prove", {"model_path": str(sketch), "timeout": timeout})
        assert "timeout" in str(refused.value)


async def test_recover_gives_the_machines_back_and_proves_it(server, model, workspace):
    code = await generated_c(server, model, workspace)
    answer = await server.call_tool("arclang_recover", {"code_path": str(code)})
    assert answer.splitlines()[0] == "Recovery of the machines of c: recovered and proven", answer
    assert "state_machine" in answer and "✓ Proven" in answer
    # Without the check it is said not to be proven.
    answer = await server.call_tool("arclang_recover", {"code_path": str(code), "check": False})
    assert "NOT proven" in answer.splitlines()[0], answer


async def test_recover_of_an_edited_code_is_a_failure(server, model, workspace):
    code = await generated_c(server, model, workspace)
    source = next(path for path in code.glob("*.c") if (code / (path.stem + ".h")).is_file())
    # A line the model does not generate: the recovered model cannot generate it again.
    source.write_text(source.read_text() + "\n/* edited by hand */\n")
    answer = await server.call_tool("arclang_recover", {"code_path": str(code)})
    assert "FAILED" in answer.splitlines()[0], answer


async def test_drift_says_no_drift_then_names_what_was_changed(server, model, workspace):
    code = await generated_c(server, model, workspace)
    answer = await server.call_tool("arclang_drift", {"model_path": str(model), "code_path": str(code)})
    assert answer.splitlines()[0].endswith(": no drift"), answer
    source = next(path for path in code.glob("*.c") if (code / (path.stem + ".h")).is_file())
    lines = source.read_text().splitlines(keepends=True)
    at = next(k for k, line in enumerate(lines) if "if (" in line and "in->" in line)
    lines[at] = lines[at].replace("if (", "if (!", 1)
    source.write_text("".join(lines))
    answer = await server.call_tool("arclang_drift", {"model_path": str(model), "code_path": str(code)})
    assert answer.splitlines()[0].endswith(": DRIFT"), answer
    assert "✗" in answer


async def test_the_paths_of_the_three_tools_stay_in_the_workspace(server, sketch, workspace, tmp_path_factory):
    outside = tmp_path_factory.mktemp("outside")
    for tool, arguments in (
        ("arclang_prove", {"model_path": str(outside / "m.arc")}),
        ("arclang_recover", {"code_path": str(outside)}),
        ("arclang_recover", {"code_path": str(workspace), "output_path": str(outside / "r.arc")}),
        ("arclang_drift", {"model_path": str(sketch), "code_path": str(outside)}),
        ("arclang_drift", {"model_path": str(sketch), "code_path": str(workspace), "hdl_path": str(outside)}),
    ):
        with pytest.raises(Exception) as refused:
            await server.call_tool(tool, arguments)
        assert "outside the workspace" in str(refused.value), (tool, arguments)


async def test_recover_reads_a_board_from_its_device_tree(server, workspace):
    tree = workspace / "board.dts"
    tree.write_text(
        '/dts-v1/;\n/ {\n    model = "Acme Board";\n    compatible = "acme,board";\n'
        '    soc {\n        compatible = "simple-bus";\n'
        '        i2c@1000 { compatible = "acme,i2c"; sensor@48 { compatible = "acme,temp"; }; };\n'
        "    };\n};\n"
    )
    answer = await server.call_tool("arclang_recover", {"code_path": str(tree), "output_path": "board.arc"})
    first = answer.splitlines()[0]
    assert first == "Board of board.dts: read from the device tree (read, not proven)", answer
    written = (workspace / "board.arc").read_text()
    assert 'protocol: "I2C, address 0x48"' in written
    assert (await server.compiler.check(workspace / "board.arc")).ok
