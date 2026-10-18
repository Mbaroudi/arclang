"""Import and transition are the compiler's; they write only where and when asked."""

import json


async def exported(server, model, workspace, export_format, name):
    await server.call_tool(
        "arclang_export", {"model_path": str(model), "format": export_format, "output_path": name}
    )
    return workspace / name


async def test_import_reads_the_sysml_text_the_compiler_exports(server, model, workspace):
    source = await exported(server, model, workspace, "sys-ml", "m.sysml")
    answer = await server.call_tool(
        "arclang_import",
        {"input_path": str(source), "format": "sysml", "output_path": "back/model.arc"},
    )
    first = answer.splitlines()[0]
    assert first.startswith("Import of m.sysml (sysml) to model.arc: succeeded"), answer
    assert "1 requirements" in answer and "4 traces" in answer, "the counts are the compiler's"
    assert "1 state machines" in answer and "1 constraints" in answer
    written = workspace / "back" / "model.arc"
    assert written.is_file()
    assert f"The written model ({written}) compiles" in answer
    assert (await server.compiler.check(written)).ok


async def test_import_relays_what_the_compiler_left_unresolved(server, model, workspace):
    """The SysML text imports standard libraries the importer does not know: it says so."""
    await exported(server, model, workspace, "sys-ml", "m.sysml")
    answer = await server.call_tool(
        "arclang_import", {"input_path": "m.sysml", "format": "sysml", "output_path": "b.arc"}
    )
    assert "designates nothing known" in answer or "left unresolved" in answer, answer


async def test_import_reads_api_records(server, model, workspace):
    source = await exported(server, model, workspace, "sysml-json", "records.json")
    answer = await server.call_tool(
        "arclang_import",
        {"input_path": str(source), "format": "sysml", "output_path": "from-records.arc"},
    )
    assert "succeeded" in answer.splitlines()[0]
    assert "2 logical components" in answer
    assert (workspace / "from-records.arc").is_file()


async def test_import_writes_nothing_but_the_output(server, model, workspace):
    source = await exported(server, model, workspace, "req-if", "m.reqif")
    before = {path for path in workspace.rglob("*")}
    await server.call_tool(
        "arclang_import", {"input_path": str(source), "format": "req-if", "output_path": "r.arc"}
    )
    assert {path for path in workspace.rglob("*")} - before == {workspace / "r.arc"}


async def test_import_of_a_missing_file_is_a_failure(server, workspace):
    answer = await server.call_tool(
        "arclang_import", {"input_path": "nope.sysml", "format": "sysml", "output_path": "x.arc"}
    )
    assert "FAILED" in answer.splitlines()[0]
    assert "cannot read" in answer
    assert not (workspace / "x.arc").exists()


async def test_transition_plan_is_the_compilers_json(server, model):
    before = model.read_text()
    answer = await server.call_tool("arclang_transition", {"model_path": str(model)})
    first, _, body = answer.partition("\n\n")
    assert first == (
        "Transition plan of braking.arc: oa-to-sa: 1 candidate(s), 1 not realized; "
        "sa-to-la: 2 candidate(s), 0 not realized; la-to-pa: 2 candidate(s), 2 not realized"
    ), answer
    steps = json.loads(body)
    assert [step["step"] for step in steps] == ["oa-to-sa", "sa-to-la", "la-to-pa"]
    assert steps[1]["candidates"][0]["realized_by"] == ["LF-001"]
    assert model.read_text() == before


async def test_transition_plan_of_one_step(server, model):
    answer = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "la-to-pa", "plan": True}
    )
    assert answer.splitlines()[0] == (
        "Transition plan of braking.arc (la-to-pa): la-to-pa: 2 candidate(s), 2 not realized"
    )


async def test_transition_step_previews_and_writes_nothing(server, model):
    before = model.read_text()
    answer = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "oa-to-sa"}
    )
    assert "NOT written" in answer.splitlines()[0]
    assert '+        function "MonitorEnvironment" {' in answer
    declared = "The compiler declared:\n+ function 'MonitorEnvironment' (SF-MonitorEnvironment)"
    assert declared in answer
    assert model.read_text() == before


async def test_transition_step_writes_when_asked_and_the_model_still_passes(server, model):
    answer = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "oa-to-sa", "write": True}
    )
    assert answer.splitlines()[0].endswith("written to braking.arc"), answer
    text = model.read_text()
    assert 'id: "SF-MonitorEnvironment"' in text
    assert 'trace "SF-MonitorEnvironment" realizes "OA-MonitorEnvironment"' in text
    assert "// Reference model served by the ArcLang MCP server." in text
    assert (await server.compiler.gate(model)).ok


async def test_transition_with_nothing_to_realize_changes_nothing(server, model):
    before = model.read_text()
    preview = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "sa-to-la", "into": "LC-001"}
    )
    assert preview.splitlines()[0].endswith("no change"), preview
    written = await server.call_tool(
        "arclang_transition",
        {"model_path": str(model), "step": "sa-to-la", "into": "LC-001", "write": True},
    )
    assert written.splitlines()[0].endswith("nothing to do, nothing written"), written
    assert model.read_text() == before


async def test_transition_into_names_the_node_that_receives_the_components(server, model):
    answer = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "la-to-pa", "into": "PN-002"}
    )
    assert "NOT written" in answer.splitlines()[0]
    assert '+            behavior_component "ThreatAssessor" {' in answer
    assert "realizes LC-001 in PN-002" in answer


async def test_transition_without_a_receiver_relays_the_compilers_refusal(server, model):
    """The reference model's `architecture physical { }` has no name: the compiler
    cannot create a container in it and says so."""
    before = model.read_text()
    answer = await server.call_tool(
        "arclang_transition", {"model_path": str(model), "step": "la-to-pa", "write": True}
    )
    assert "FAILED" in answer.splitlines()[0]
    assert model.read_text() == before


async def test_transition_write_needs_a_step(server, model):
    import pytest

    before = model.read_text()
    with pytest.raises(ValueError, match="needs a 'step'"):
        await server.call_tool("arclang_transition", {"model_path": str(model), "write": True})
    with pytest.raises(ValueError, match="needs a 'step'"):
        await server.call_tool(
            "arclang_transition",
            {"model_path": str(model), "step": "oa-to-sa", "plan": True, "write": True},
        )
    assert model.read_text() == before


async def test_transition_arguments_are_the_clis(server, model):
    import pytest

    with pytest.raises(ValueError, match="'step' must be one of"):
        await server.call_tool("arclang_transition", {"model_path": str(model), "step": "pa-to-oa"})
    with pytest.raises(ValueError, match="'format' must be one of"):
        await server.call_tool(
            "arclang_import", {"input_path": "a", "format": "docx", "output_path": "b.arc"}
        )
