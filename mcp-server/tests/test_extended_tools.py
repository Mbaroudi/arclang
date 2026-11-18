"""The tools that gained options: what each new one does, with the real binary where nothing heavy runs."""

import json
import shutil
from pathlib import Path

import pytest

from arclang_mcp.compiler.wrapper import DIRECTORY_EXPORT_FORMATS, EXPORT_FORMATS, Run
from arclang_mcp.tools import cli
from arclang_mcp.tools.common import input_schema

from test_command_lines import BASE, CHOICE, CONTEXT

REPOSITORY = Path(__file__).resolve().parents[2]
SKETCHES = REPOSITORY / "tests" / "fixtures" / "sketches"


# --------------------------------------------------------------------- recover


@pytest.fixture
def firmware(workspace):
    """The binary of a sketch, as the Arduino IDE builds it for an UNO."""
    shutil.copy(SKETCHES / "uno_lamp.hex", workspace / "uno_lamp.hex")
    return workspace / "uno_lamp.hex"


async def test_recover_of_a_binary_asks_for_no_proof_and_claims_none(server, firmware, recorded):
    recorded.reply = Run(
        (),
        0,
        'correspondence "uno_lamp.hex" { binary: "uno_lamp.hex" }\n',
        "Recovered from uno_lamp.hex: 2 function(s) placed of 31 found in the machine code\n"
        "  a proposal, not a proof: write the machine, then `arclang prove` decides\n",
    )
    answer = await server.call_tool("arclang_recover", {"code_path": "uno_lamp.hex"})
    assert "--check" not in recorded.last, "the CLI proves nothing of a binary: --check is not passed"
    first = answer.splitlines()[0]
    assert first == (
        "Reading of the binary uno_lamp.hex: read from the binary, a correspondence proposed (NOT proven)"
    )
    assert "recovered and proven" not in answer
    # The same with `check: true` said aloud: still no proof asked, still none claimed.
    answer = await server.call_tool("arclang_recover", {"code_path": "uno_lamp.hex", "check": True})
    assert "--check" not in recorded.last and "NOT proven" in answer.splitlines()[0]


async def test_recover_of_a_binary_with_its_model_looks_for_the_step(server, firmware, model, recorded):
    recorded.reply = Run(
        (), 0, "", "Recovered from uno_lamp.hex: the step of the model is the function at 0x0150\n"
        "  a proposal, not a proof: put the correspondence in the model, then `arclang prove` decides\n",
    )
    answer = await server.call_tool(
        "arclang_recover",
        {"code_path": "uno_lamp.hex", "model_path": "braking.arc", "keep_dir": "lift", "output_path": "found.arc"},
    )
    assert any(a.startswith("--model=") and a.endswith("braking.arc") for a in recorded.last)
    assert any(a.startswith("--keep=") for a in recorded.last) and "--check" not in recorded.last
    assert answer.splitlines()[0].endswith("the step of the model placed in the binary (a proposal, NOT proven)")


async def test_recover_of_a_binary_with_the_real_compiler_never_says_proven(server, firmware, monkeypatch):
    """Whatever this machine has (Ghidra or not), the first line does not say the binary is proven."""
    monkeypatch.delenv("ARCLANG_GHIDRA", raising=False)
    monkeypatch.setenv("PATH", "/usr/bin:/bin")
    answer = await server.call_tool("arclang_recover", {"code_path": "uno_lamp.hex", "timeout": 60})
    first = answer.splitlines()[0]
    assert first.startswith("Reading of the binary uno_lamp.hex: ")
    assert "FAILED" in first and "Ghidra" in answer, answer


async def test_recover_refuses_what_means_nothing_for_what_it_reads(server, firmware, workspace, recorded):
    (workspace / "board.dts").write_text("/dts-v1/;\n/ { };\n")
    for arguments, mention in (
        ({"code_path": "uno_lamp.hex", "hdl_path": "vhdl"}, "'hdl_path' mean(s) nothing for a binary"),
        ({"code_path": "uno_lamp.hex", "view_path": "v.html"}, "'view_path' mean(s) nothing for a binary"),
        ({"code_path": "board.dts", "model_path": "braking.arc"}, "mean(s) nothing for a device tree"),
        ({"code_path": "c", "turns": 2}, "'turns' goes with 'propose: true'"),
        ({"code_path": "c", "keep_dir": "k"}, "mean(s) nothing for generated C"),
        ({"code_path": "c", "open": True}, "'open' is not an argument"),
    ):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_recover", arguments)
        assert mention in str(refused.value), arguments
    assert recorded.commands == []


async def test_recover_draws_the_recovered_model_and_never_opens_it(server, model, workspace):
    code = workspace / "c"
    assert (await server.compiler.run(["export", str(model), "-f", "c", "-o", str(code)])).ok
    answer = await server.call_tool("arclang_recover", {"code_path": "c", "view_path": "view/recovered.html"})
    assert answer.splitlines()[0] == "Recovery of the machines of c: recovered and proven", answer
    page = (workspace / "view" / "recovered.html").read_text()
    assert "<html" in page.lower() and "BrakingModes" in page


async def test_a_proposal_needs_a_proposer_the_server_was_given(server, workspace, recorded):
    shutil.copy(SKETCHES / "toggle.ino", workspace / "toggle.ino")
    with pytest.raises(Exception) as refused:
        await server.call_tool("arclang_recover", {"code_path": "toggle.ino", "propose": True, "with": "cat answer.txt"})
    assert "ARCLANG_MCP_ALLOW_COMMANDS=1" in str(refused.value) and "ARCLANG_PROPOSER" in str(refused.value)
    assert recorded.commands == []
    for turns in (0, 13):
        with pytest.raises(Exception, match="'turns' is from 1 to 12"):
            await server.call_tool("arclang_recover", {"code_path": "toggle.ino", "propose": True, "turns": turns})
    # Without `with`, the proposer is the one of the server's environment: none is written.
    await server.call_tool(
        "arclang_recover",
        {"code_path": "toggle.ino", "propose": True, "turns": 2, "timeout": 100, "output_path": "out/toggle.arc"},
    )
    assert recorded.last[:4] == ["recover", "--propose", "--turns=2", "--timeout=100"]
    assert not any(a.startswith("--with") or a == "--open" or a == "--check" for a in recorded.last)
    assert recorded.timeouts[-1] == 2 * (600 + 400) + 120


async def test_a_proposal_is_proven_or_it_is_not(server, workspace, recorded, allow_commands):
    shutil.copy(SKETCHES / "toggle.ino", workspace / "toggle.ino")
    recorded.reply = Run((), 0, '✓ Proven (CBMC, Bitwuzla) toggle.ino is state_machine "Lamp"\n  Output: toggle.arc\n', "")
    answer = await server.call_tool(
        "arclang_recover",
        {"code_path": "toggle.ino", "propose": True, "with": "my-proposer --model x", "view_path": "p.html"},
    )
    assert "--with=my-proposer --model x" in recorded.last and "--open" not in recorded.last
    assert answer.splitlines()[0] == "Proposal of a model of toggle.ino: proposed, then proven"
    recorded.reply = Run(
        (), 1, "", "Error: Compilation error: no model of toggle.ino was proven in 4 turn(s); nothing is written."
    )
    answer = await server.call_tool("arclang_recover", {"code_path": "toggle.ino", "propose": True})
    assert answer.splitlines()[0] == (
        "Proposal of a model of toggle.ino: NOT PROVEN: no proposal was proven, nothing is written"
    )


async def test_a_proposal_without_any_proposer_fails_and_says_how(server, workspace):
    shutil.copy(SKETCHES / "toggle.ino", workspace / "toggle.ino")
    answer = await server.call_tool("arclang_recover", {"code_path": "toggle.ino", "propose": True})
    assert "FAILED" in answer.splitlines()[0] and "--propose needs a proposer" in answer
    assert not (workspace / "toggle.arc").exists()


async def test_the_description_of_recover_names_what_it_reads(server):
    description = server.tools_by_name["arclang_recover"].description
    for word in (".dts", ".hex", ".ino", "PROPOSED", "Nothing is proven", "never opened"):
        assert word in description, word


async def test_prove_and_drift_give_their_json(server, model, workspace, recorded):
    await server.call_tool("arclang_prove", {"model_path": "braking.arc", "as_json": True, "keep_dir": "proof"})
    assert "--json" in recorded.last and any(a.startswith("--keep=") for a in recorded.last)
    recorded.reply = Run((), 0, '{"correspondences": []}', "")
    answer = await server.call_tool("arclang_prove", {"model_path": "braking.arc", "as_json": True})
    # No word of the compiler says proven: the first line does not either.
    assert "proven" not in answer.splitlines()[0].replace("read the outcome", "")


async def test_drift_as_json_with_the_real_compiler(server, model, workspace):
    code = workspace / "c"
    assert (await server.compiler.run(["export", str(model), "-f", "c", "-o", str(code)])).ok
    answer = await server.call_tool(
        "arclang_drift", {"model_path": str(model), "code_path": "c", "as_json": True}
    )
    assert answer.splitlines()[0].endswith(": no drift"), answer
    assert json.loads(answer.split("\n\n", 1)[1])["identical"] is True
    source = code / "emergency_braking.c"
    source.write_text(source.read_text() + "\n/* by hand */\n")
    answer = await server.call_tool(
        "arclang_drift", {"model_path": str(model), "code_path": "c", "as_json": True}
    )
    assert answer.splitlines()[0].endswith(": DRIFT"), answer


# ---------------------------------------------------------------------- impact


async def test_impact_of_several_elements_bounded(server, model):
    answer = await server.call_tool(
        "arclang_impact",
        {"model_path": str(model), "elements": ["LC-001", "PN-002"], "depth": 1, "direction": "downstream",
         "kinds": ["deployment", "satisfies"], "format": "json", "gate_standard": "DO178C"},
    )
    assert answer.splitlines()[0] == "Impact of the change in braking.arc: succeeded", answer
    document = json.loads(answer.split("\n\n", 1)[1])
    assert {seed["id"] for seed in document["seeds"]} == {"LC-001", "PN-002"}
    assert all(item["distance"] == 1 for item in document["impacted"])


async def test_impact_of_one_element_is_what_it_was(server, model):
    answer = await server.call_tool("arclang_impact", {"model_path": str(model), "element": "REQ-BRK-001"})
    assert answer.splitlines()[0] == "Impact of 'REQ-BRK-001' in braking.arc: succeeded"
    assert "LC-001" in answer and "LC-002" in answer


async def test_impact_of_a_proposed_change_writes_nothing(server, model):
    before = model.read_text()
    change = {"op": "set", "element": "LF-002", "key": "latency", "value": "45 ms"}
    answer = await server.call_tool("arclang_impact", {"model_path": str(model), "change": change, "format": "md"})
    assert answer.splitlines()[0].endswith(": succeeded"), answer
    # The what-if breaks the 50 ms budget: the gate after it fails, and the answer says so.
    assert "CST-001" in answer and "FAIL" in answer
    assert model.read_text() == before
    several = await server.call_tool(
        "arclang_impact",
        {"model_path": str(model), "change": [change, {"op": "rename", "element": "LC-002", "name": "Brakes"}]},
    )
    assert several.splitlines()[0].endswith(": succeeded") and model.read_text() == before


async def test_impact_of_a_deletion_the_model_does_not_survive_is_a_failure(server, model):
    before = model.read_text()
    answer = await server.call_tool("arclang_impact", {"model_path": str(model), "delete": "LC-001"})
    assert "FAILED" in answer.splitlines()[0] and "does not compile" in answer
    answer = await server.call_tool("arclang_impact", {"model_path": str(model), "delete": "PP-001"})
    assert answer.splitlines()[0].endswith(": succeeded"), answer
    assert model.read_text() == before


async def test_impact_writes_a_file_only_where_told(server, model, workspace):
    answer = await server.call_tool(
        "arclang_impact",
        {"model_path": str(model), "element": "LC-001", "format": "xlsx", "output_path": "out/impact.xlsx"},
    )
    assert (workspace / "out" / "impact.xlsx").read_bytes()[:2] == b"PK" and "Written to" in answer
    for arguments, mention in (
        ({"model_path": str(model)}, "say what changes"),
        ({"model_path": str(model), "element": "LC-001", "format": "xlsx"}, "give 'output_path'"),
        ({"model_path": str(model), "element": "LC-001", "current_path": "b.arc"}, "goes with 'since'"),
        ({"model_path": str(model), "change": "@/etc/passwd"}, "an object) or a list"),
        ({"model_path": str(model), "change": "-"}, "an object) or a list"),
        ({"model_path": str(model), "element": "LC-001", "kinds": ["friendship"]}, "must be one of"),
        ({"model_path": str(model), "element": "LC-001", "depth": 0}, "'depth' must be at least 1"),
    ):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_impact", arguments)
        assert mention in str(refused.value), arguments


# ---------------------------------------------------------------------- export


async def test_export_fills_a_directory_and_lists_it(server, model, workspace):
    answer = await server.call_tool(
        "arclang_export", {"model_path": str(model), "format": "c", "output_path": "build/c"}
    )
    assert answer.splitlines()[0] == "Export of braking.arc to c: succeeded", answer
    assert f"Written to {workspace / 'build' / 'c'}:" in answer
    assert "- emergency_braking.c (" in answer and "- arclang_io.h (" in answer
    assert (workspace / "build" / "c" / "emergency_braking.h").is_file()


async def test_a_directory_export_without_output_path_keeps_nothing(server, model, workspace):
    answer = await server.call_tool("arclang_export", {"model_path": str(model), "format": "modelica"})
    assert "Not kept (no output_path" in answer and "- EmergencyBraking.mo (" in answer
    assert sorted(path.name for path in workspace.iterdir()) == ["braking.arc"]
    split = await server.call_tool(
        "arclang_export",
        {"model_path": str(model), "format": "modelica", "split": True, "no_placement": True, "output_path": "mo"},
    )
    assert "package.mo" in split, split
    assert "Placement(" not in "".join(p.read_text() for p in (workspace / "mo").rglob("*.mo"))


async def test_export_of_the_new_single_file_formats(server, model, workspace):
    geojson = await server.call_tool("arclang_export", {"model_path": str(model), "format": "geojson"})
    assert '"FeatureCollection"' in geojson
    connector = await server.call_tool(
        "arclang_export",
        {"model_path": str(model), "format": "simulink-connector", "output_path": "sl/braking",
         "strategy": "state-machines", "leaf_functions": "subsystem", "port_naming": "CapellaElement"},
    )
    assert connector.splitlines()[0].endswith(": succeeded"), connector
    assert (workspace / "sl" / "braking.mlcore").is_file() and (workspace / "sl" / "braking.m").is_file()
    unkept = await server.call_tool("arclang_export", {"model_path": str(model), "format": "simulink-connector"})
    assert "Not kept (no output_path); the export is:" in unkept and ".mlcore" in unkept


async def test_an_export_the_model_has_nothing_for_is_the_compilers_refusal(server, model):
    for export_format, reason in (
        ("zephyr", "no target"),
        ("vhdl", "no state_machine is allocated to programmable logic"),
        ("step", "no element has a `shape`"),
    ):
        answer = await server.call_tool("arclang_export", {"model_path": str(model), "format": export_format})
        assert "FAILED" in answer.splitlines()[0] and reason in answer, answer


async def test_export_writes_the_options_of_its_format(server, recorded):
    await server.call_tool(
        "arclang_export",
        {"model_path": "braking.arc", "format": "zephyr", "target": "zephyr/litex_vexriscv",
         "binding": "Wiper task", "output_path": "build/app"},
    )
    assert {"--format=zephyr", "--target=zephyr/litex_vexriscv", "--binding=Wiper task"} <= set(recorded.last)
    await server.call_tool(
        "arclang_export",
        {"model_path": "braking.arc", "format": "simulink-connector", "elements": ["SF-001", "SF-002"]},
    )
    assert "--element=SF-001,SF-002" in recorded.last
    # The FMU compiles C on this computer: its command line is read here, the compiler not run.
    await server.call_tool("arclang_export", {"model_path": "braking.arc", "format": "fmu", "output_path": "fmu"})
    assert "--format=fmu" in recorded.last and recorded.timeouts[-1] >= 120


async def test_pdf_is_not_offered_because_the_cli_refuses_it(server, model):
    assert "pdf" not in EXPORT_FORMATS and set(DIRECTORY_EXPORT_FORMATS) <= set(EXPORT_FORMATS)
    with pytest.raises(Exception, match="must be one of"):
        await server.call_tool("arclang_export", {"model_path": str(model), "format": "pdf"})


# -------------------------------------------------------------------- diagrams


async def test_the_new_views_and_the_options_of_the_viewer(server, model, workspace):
    answer = await server.call_tool(
        "arclang_viewpoint_diagrams",
        {"model_path": str(model), "view": "cesam", "theme": "print-grayscale", "output_path": "out/cesam.html"},
    )
    assert answer.splitlines()[0].endswith(": succeeded"), answer
    assert "✓ req" in answer and "✓ cdb" in answer
    for view in ("req", "cdb", "pbs", "cap", "par"):
        answer = await server.call_tool(
            "arclang_viewpoint_diagrams",
            {"model_path": str(model), "view": view, "format": "viewpoints", "output_path": f"out/{view}.json"},
        )
        # A view the reference model has nothing for is the compiler's refusal, never an unknown view.
        drawn = answer.splitlines()[0].endswith(": succeeded")
        assert drawn or f"declares nothing to draw in the {view} view" in answer, (view, answer)
        assert drawn == (view in ("req", "cdb", "par")), (view, answer)
    layout = workspace / "braking.layout.json"
    layout.write_text('{"arclang_layout": "1", "views": {}}')
    answer = await server.call_tool(
        "arclang_viewpoint_diagrams",
        {"model_path": str(model), "format": "elk", "layout_path": "braking.layout.json", "output_path": "out/elk.json"},
    )
    assert answer.splitlines()[0].endswith(": succeeded"), answer


async def test_a_theme_is_a_preset_or_a_file_of_the_workspace(server, model, workspace, tmp_path_factory):
    fixture = json.loads((REPOSITORY / "tests" / "fixtures" / "themes" / "good-custom.json").read_text())
    (workspace / "arclang-theme.json").write_text(json.dumps(fixture["theme"]))
    answer = await server.call_tool(
        "arclang_viewpoint_diagrams",
        {"model_path": str(model), "theme": "arclang-theme.json", "output_path": "out/themed.html"},
    )
    assert answer.splitlines()[0].endswith(": succeeded"), answer
    with pytest.raises(Exception, match="goes with 'format: html'"):
        await server.call_tool(
            "arclang_viewpoint_diagrams", {"model_path": str(model), "format": "elk", "theme": "print-grayscale"}
        )
    outside = tmp_path_factory.mktemp("elsewhere") / "theme.json"
    with pytest.raises(Exception, match="outside the workspace"):
        await server.call_tool("arclang_viewpoint_diagrams", {"model_path": str(model), "theme": str(outside)})


async def test_a_text_diagram_takes_a_title(server, model, workspace):
    await server.call_tool(
        "arclang_generate_diagram",
        {"model_path": str(model), "diagram_type": "mermaid", "title": "Braking, as designed", "output_path": "d.mmd"},
    )
    assert (workspace / "d.mmd").is_file()


async def test_no_diagram_tool_can_open_a_browser(server, recorded):
    for tool in ("arclang_viewpoint_diagrams", "arclang_generate_diagram"):
        with pytest.raises(Exception, match="'open' is not an argument"):
            await server.call_tool(tool, {"model_path": "braking.arc", "diagram_type": "mermaid", "open": True})
    await server.call_tool("arclang_viewpoint_diagrams", {"model_path": "braking.arc"})
    await server.call_tool("arclang_generate_all_diagrams", {"model_path": "braking.arc", "output_dir": "d"})
    assert not any("--open" in command for command in recorded.commands)


# ------------------------------------------------------------------------ edit


@pytest.fixture
def without_path(model):
    """The reference model less its physical path.

    With a `physical_path` that `involves:` a link in the model, the compiler
    refuses every `duplicate` ("would lose a relationship of the model"), even of
    an element the path does not touch: reported, and kept out of these tests.
    """
    text = model.read_text()
    start = text.index('        physical_path "ThreatPath" {')
    model.write_text(text[:start] + text[text.index("        }\n", start) + len("        }\n"):])
    return model


async def test_the_new_edit_operations_edit_the_model(server, without_path):
    model = without_path
    requests = [
        {"op": "duplicate", "element": "LC-002"},
        {"op": "reconnect", "kind": "PhysicalLink", "from": "PN-001", "to": "PN-002",
         "new_from": "PN-002", "new_to": "PN-001"},
        {"op": "wrap_invariant", "scenario": "EmergencyStop", "keyword": "mode", "lifeline": "LC-002",
         "name": "Braking", "first": "critical threat", "last": "critical threat"},
    ]
    answer = await server.call_tool("arclang_edit", {"path": str(model), "requests": requests, "write": True})
    assert answer.splitlines()[0] == "Edit of braking.arc (3 requests): written to braking.arc", answer
    assert "1. duplicate LC-002 beside it" in answer
    assert "2. reconnect the PhysicalLink from PN-001 to PN-002: its source to PN-002 and its target to PN-001" in answer
    assert '3. assert the mode "Braking" on LC-002 over the messages critical threat...critical threat of EmergencyStop' in answer
    text = model.read_text()
    assert 'component "BrakeController 2"' in text and 'id: "LC-003"' in text
    assert 'from: "PN-002"' in text and 'to: "PN-001"' in text
    assert (await server.compiler.check(model)).ok


async def test_duplicate_many_copies_what_joins_the_copies(server, without_path):
    model = without_path
    answer = await server.call_tool(
        "arclang_edit",
        {"path": str(model), "requests": [{"op": "duplicate_many", "elements": ["SF-001", "SF-002"]}]},
    )
    assert "accepted, NOT written" in answer.splitlines()[0], answer
    assert "duplicate SF-001, SF-002 beside them" in answer
    assert '+            id: "SF-003"' in answer and '+            id: "SF-004"' in answer
    # The exchange between the two is copied too, joined to the copies.
    assert '+            from: "SF-003.threat_level"' in answer and '+            to: "SF-004.threat_level"' in answer


async def test_a_reconnection_the_compiler_refuses_changes_nothing(server, model):
    before = model.read_text()
    answer = await server.call_tool(
        "arclang_edit",
        {"path": str(model), "write": True, "requests": [
            {"op": "reconnect", "kind": "ComponentExchange", "from": "LC-001", "to": "LC-002", "new_to": "LC-001"}
        ]},
    )
    assert "refused at request 1" in answer and "Nothing was changed." in answer
    assert model.read_text() == before


# ---------------------------------------------------- safety, trace, check, build


async def test_safety_gives_the_fault_trees_and_the_report(server, model, workspace):
    tree = await server.call_tool("arclang_safety_check", {"model_path": str(model), "standard": "iso26262", "fta": True})
    assert tree.splitlines()[0] == "Safety content of braking.arc (iso26262): fta printed (not a compliance verdict)"
    assert "H-001" in tree and "no failure mode leads to it" in tree
    written = await server.call_tool(
        "arclang_safety_check",
        {"model_path": str(model), "standard": "iso26262", "report": True, "output_path": "out/safety.md"},
    )
    assert "Written to" in written and (workspace / "out" / "safety.md").read_text().startswith("# Safety report")
    # The reference model declares no FMEA entry: the compiler says so, and that is a failure.
    fmea = await server.call_tool("arclang_safety_check", {"model_path": str(model), "standard": "iso26262", "fmea": True})
    assert "FAILED" in fmea.splitlines()[0] and "no fmea entries" in fmea


async def test_trace_lists_the_suspects(server, model):
    answer = await server.call_tool("arclang_trace_analysis", {"model_path": str(model), "suspects": True})
    assert answer.splitlines()[0] == (
        "Suspect traces of braking.arc: 0 of 4 trace(s) and 0 of 1 verification(s) reviewed, 0 suspect"
    )


async def test_a_review_is_a_diff_until_written_and_a_change_makes_it_suspect(server, model):
    before = model.read_text()
    preview = await server.call_tool("arclang_trace_analysis", {"model_path": str(model), "review": True, "from": "LC-001"})
    assert "accepted, NOT written" in preview.splitlines()[0], preview
    assert preview.count("+") >= 1 and "reviewed:" in preview and model.read_text() == before
    written = await server.call_tool(
        "arclang_trace_analysis", {"model_path": str(model), "review": True, "from": "LC-001", "write": True}
    )
    assert written.splitlines()[0] == "Review of the traces of braking.arc: written to braking.arc", written
    assert model.read_text().count("reviewed:") == 1
    clean = await server.call_tool("arclang_trace_analysis", {"model_path": str(model), "suspects": True})
    assert "1 of 4 trace(s)" in clean.splitlines()[0] and "0 suspect" in clean.splitlines()[0]
    # One end of the reviewed trace changes: the review no longer covers it.
    model.write_text(model.read_text().replace(
        'description: "The system shall brake when a collision is imminent"',
        'description: "The system shall brake hard when a collision is imminent"',
    ))
    suspect = await server.call_tool("arclang_trace_analysis", {"model_path": str(model), "suspects": True})
    assert "1 suspect" in suspect.splitlines()[0], suspect


async def test_trace_refuses_a_write_that_is_no_review(server, model):
    for arguments, mention in (
        ({"write": True}, "go with 'review: true'"),
        ({"from": "LC-001"}, "go with 'review: true'"),
        ({"review": True, "suspects": True}, "ask 'suspects' or 'matrix' apart"),
    ):
        with pytest.raises(Exception) as refused:
            await server.call_tool("arclang_trace_analysis", {"model_path": str(model), **arguments})
        assert mention in str(refused.value)


async def test_check_by_another_method_and_with_safety(server, model):
    arcadia = await server.call_tool("arclang_validate", {"model_path": str(model), "as_json": True})
    cesam = await server.call_tool(
        "arclang_validate", {"model_path": str(model), "as_json": True, "method": "cesam", "safety": True}
    )
    rules = lambda answer: {f["rule"] for f in json.loads(answer.split("\n\n", 1)[1])["findings"]}  # noqa: E731
    assert not any(rule.startswith("cesam.") for rule in rules(arcadia))
    assert any(rule.startswith("cesam.") for rule in rules(cesam)), cesam
    with pytest.raises(Exception, match="two answers"):
        await server.call_tool("arclang_validate", {"model_path": str(model), "as_json": True, "format": "sarif"})


async def test_compile_to_another_target(server, model, workspace):
    answer = await server.call_tool(
        "arclang_compile", {"model_path": str(model), "target": "xml", "output_path": "out/braking.xml"}
    )
    assert answer.splitlines()[0] == "Compilation of braking.arc: succeeded", answer
    assert (workspace / "out" / "braking.xml").read_text().lstrip().startswith("<")


async def test_the_metamodel_as_json(server):
    answer = await server.call_tool("arclang_metamodel", {"format": "json"})
    assert {"kinds", "relationships"} <= set(json.loads(answer))


# ------------------------------------------------------------- paths, every one


def path_arguments(server):
    """Every argument of every tool that is a path, by the way the tools name them."""
    for tool in server.tools:
        for argument, declared in input_schema(tool)["properties"].items():
            is_path = (
                argument in ("path", "directory", "output_dir", "out_dir")
                or argument.endswith(("_path", "_dir"))
            )
            if is_path and declared.get("type") == "string":
                yield tool.name, argument


async def test_every_path_argument_of_every_tool_stays_in_the_workspace(
    server, recorded, allow_commands, tmp_path_factory
):
    outside = tmp_path_factory.mktemp("outside")
    checked = set()
    for tool, argument in path_arguments(server):
        if tool in cli.WITHOUT_SUBCOMMAND:
            continue
        choices = [CHOICE[c] for c in cli.COVERS[tool] if c in CHOICE] or [{}]
        refused_once = False
        for choice in choices:
            call = {**BASE.get(tool, {"model_path": "braking.arc"}), **choice, **CONTEXT.get((tool, argument), {})}
            call = {key: value for key, value in call.items() if value is not None}
            call[argument] = str(outside / "somewhere")
            recorded.commands.clear()
            try:
                await server.call_tool(tool, call)
            except Exception as refused:
                if "outside the workspace" in str(refused):
                    refused_once = True
                continue
            leaked = [c for c in recorded.commands if any(str(outside) in a for a in c)]
            assert not leaked, f"{tool}.{argument} reached the compiler with a path outside: {leaked}"
        assert refused_once, f"{tool}.{argument}: a path outside the workspace was not refused"
        checked.add((tool, argument))
    assert len(checked) > 70, len(checked)
    # The lists of models are paths too.
    for tool in ("arclang_report", "arclang_sarif", "arclang_ci_run"):
        with pytest.raises(Exception, match="outside the workspace"):
            await server.call_tool(tool, {**BASE.get(tool, {}), "models": [str(outside / "m.arc")]})
        with pytest.raises(Exception, match="outside the workspace"):
            await server.call_tool(tool, {**BASE.get(tool, {}), "models": ["../../m.arc"]})
