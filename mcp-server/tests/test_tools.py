"""Each tool returns what the compiler says, and says so when the compiler fails."""

import json

import pytest

from arclang_mcp.compiler.wrapper import DIAGRAM_TYPES, EXPORT_FORMATS, ArcLangCompiler
from arclang_mcp.tools.common import InvalidArguments, input_schema


async def test_every_listed_tool_has_a_handler_and_a_schema(server):
    names = [tool.name for tool in server.tools]
    assert len(names) == len(set(names))
    assert set(names) == set(server.handlers)
    for tool in server.tools:
        arguments = input_schema(tool)
        assert arguments["type"] == "object"
        assert set(arguments["required"]) <= set(arguments["properties"])


async def test_unknown_tool_is_an_error(server):
    with pytest.raises(ValueError, match="Unknown tool"):
        await server.call_tool("arclang_nope", {})


async def test_validate_accepts_the_reference_model(server, model):
    answer = await server.call_tool("arclang_validate", {"model_path": "braking.arc"})
    assert answer.startswith("Check of braking.arc: compiles; 0 compilation warning(s)")
    assert "No compilation errors" in answer


async def test_validate_lists_no_untraced_element_in_a_traced_model(server, model):
    answer = await server.call_tool("arclang_validate", {"model_path": str(model)})
    assert "0 traceability warning(s)" in answer.splitlines()[0]
    assert "has no trace" not in answer


async def test_validate_names_the_requirement_no_trace_reaches(server, model, workspace):
    untraced = workspace / "untraced.arc"
    untraced.write_text(
        "\n".join(line for line in model.read_text().splitlines() if "REQ-BRK-001\" {" not in line)
    )
    answer = await server.call_tool("arclang_validate", {"model_path": str(untraced)})
    assert "Requirement REQ-BRK-001 has no trace" in answer


async def test_compile_writes_the_model_as_json(server, model, workspace):
    await server.call_tool("arclang_compile", {"model_path": str(model), "output_path": "m.json"})
    compiled = json.loads((workspace / "m.json").read_text())
    assert isinstance(compiled, dict) and compiled


@pytest.mark.parametrize(
    "tool, arguments, mention",
    [
        ("arclang_format", {"path": ""}, "'path' must not be empty"),
        ("arclang_set_attribute", {"model_path": "a.arc", "element": "", "key": "k", "value": "1"}, "'element' must not be empty"),
        ("arclang_suggest_architecture", {"requirements": [1]}, "'requirements'[0] must be a string"),
        ("arclang_info", {"model_path": float("nan")}, "must be a string"),
        ("arclang_compile", {"model_path": "a.arc", "optimize": 1}, "'optimize' must be a boolean"),
    ],
)
async def test_arguments_are_checked_to_the_leaves(server, tool, arguments, mention):
    with pytest.raises(InvalidArguments) as refusal:
        await server.call_tool(tool, arguments)
    assert mention in str(refusal.value)


async def test_export_without_output_path_writes_nothing_and_returns_the_content(server, model, workspace):
    before = {p for p in workspace.rglob("*")}
    answer = await server.call_tool(
        "arclang_export", {"model_path": str(model), "format": "markdown"}
    )
    assert "Not written (no output_path)" in answer
    assert "REQ-BRK-001" in answer
    assert {p for p in workspace.rglob("*")} == before


async def test_a_version_probe_that_fails_is_not_remembered(server, workspace):
    server.compiler.binary_path = str(workspace / "no-such-binary")
    assert await server.compiler_note() == ""
    assert server._compiler_note is None


async def test_validate_reports_the_error_with_its_position(server, broken):
    answer = await server.call_tool("arclang_validate", {"model_path": str(broken)})
    assert "FAILED" in answer
    assert "line 3" in answer


async def test_compile_writes_the_output_asked_for(server, model, workspace):
    answer = await server.call_tool(
        "arclang_compile", {"model_path": str(model), "output_path": "out/compiled"}
    )
    assert "succeeded" in answer
    assert (workspace / "out" / "compiled").is_file()


async def test_compile_failure_is_not_a_success(server, broken):
    answer = await server.call_tool("arclang_compile", {"model_path": str(broken)})
    assert "FAILED" in answer and "succeeded" not in answer


async def test_gate_passes_on_the_reference_model(server, model):
    answer = await server.call_tool("arclang_gate", {"model_path": str(model)})
    assert answer.startswith("Production gate of braking.arc: PASS")


async def test_gate_fails_with_its_blockers(server, model):
    model.write_text(
        model.read_text().replace('verifies: ["REQ-BRK-001"]', 'verifies: ["REQ-OTHER"]')
    )
    answer = await server.call_tool("arclang_gate", {"model_path": str(model)})
    assert answer.splitlines()[0] == "Production gate of braking.arc: FAIL, 3 blocker(s)"
    assert "✗ 3 blocker(s):" in answer, "the count must be the compiler's"
    assert "[requirements.verification]" in answer


async def test_gate_on_a_model_that_does_not_compile_gives_no_verdict(server, broken):
    answer = await server.call_tool("arclang_gate", {"model_path": str(broken)})
    first = answer.splitlines()[0]
    assert "PASS" not in first and "blocker" not in first
    assert "FAILED" in first


async def test_diff_reports_a_rename_as_a_modification(server, model, workspace):
    renamed = workspace / "renamed.arc"
    renamed.write_text(
        model.read_text().replace('component "BrakeController"', 'component "BrakeCommander"')
    )
    answer = await server.call_tool(
        "arclang_diff", {"old_path": str(model), "new_path": str(renamed)}
    )
    assert "1 modified" in answer.splitlines()[0]
    assert "BrakeCommander" in answer


@pytest.mark.parametrize(
    "old, new",
    [
        ("latency: 10 ms", "latency: 12 ms"),
        ('severity: "S3"', 'severity: "S2"'),
        ("Computes and commands the braking force", "Commands the brakes"),
    ],
)
async def test_diff_reports_an_attribute_change(server, model, workspace, old, new):
    changed = workspace / "changed.arc"
    source = model.read_text()
    assert old in source
    changed.write_text(source.replace(old, new))
    answer = await server.call_tool(
        "arclang_diff", {"old_path": str(model), "new_path": str(changed)}
    )
    assert "no semantic change" not in answer.splitlines()[0]
    assert new.split(": ")[-1].strip('"') in answer


async def test_diff_of_a_file_with_itself(server, model):
    answer = await server.call_tool(
        "arclang_diff", {"old_path": str(model), "new_path": str(model)}
    )
    assert "no semantic change" in answer.splitlines()[0]


async def test_impact_follows_traces(server, model):
    answer = await server.call_tool(
        "arclang_impact", {"model_path": str(model), "element": "REQ-BRK-001"}
    )
    assert "LC-002" in answer and "succeeded" in answer


async def test_impact_of_an_unknown_element_fails(server, model):
    answer = await server.call_tool("arclang_impact", {"model_path": str(model), "element": "NOPE"})
    assert "FAILED" in answer


async def test_export_formats_are_those_of_the_cli(server):
    help_text = (await server.compiler.run(["export", "--help"])).stdout
    for export_format in EXPORT_FORMATS:
        assert f"- {export_format}" in help_text, export_format


async def test_diagram_types_are_those_of_the_cli(server):
    help_text = (await server.compiler.run(["diagram", "--help"])).stdout
    for diagram_type in DIAGRAM_TYPES:
        assert f"- {diagram_type}" in help_text, diagram_type


async def test_export_sysml_is_returned(server, model):
    answer = await server.call_tool(
        "arclang_export", {"model_path": str(model), "format": "sys-ml"}
    )
    assert "succeeded" in answer
    assert "package EmergencyBraking" in answer


async def test_export_to_a_file(server, model, workspace):
    await server.call_tool(
        "arclang_export",
        {"model_path": str(model), "format": "sysml-json", "output_path": "out/model.json"},
    )
    assert isinstance(json.loads((workspace / "out" / "model.json").read_text()), (list, dict))


async def test_export_refuses_an_unknown_format(server, model):
    with pytest.raises(ValueError, match="'format' must be one of"):
        await server.call_tool("arclang_export", {"model_path": str(model), "format": "docx"})


async def test_metamodel_of_one_kind(server):
    answer = await server.call_tool("arclang_metamodel", {"kind": "logicalcomponent"})
    keys = {attribute["key"] for attribute in json.loads(answer)["attributes"]}
    assert "multiplicity" in keys


async def test_metamodel_unknown_kind_lists_the_known_ones(server):
    answer = await server.call_tool("arclang_metamodel", {"kind": "Widget"})
    assert answer.startswith("No element kind 'Widget'") and "Requirement" in answer


async def test_review_sends_nothing_by_default(server, model):
    answer = await server.call_tool("arclang_review", {"model_path": str(model)})
    assert "dry run, nothing was sent" in answer.splitlines()[0]
    assert "Nothing was sent (--dry-run)" in answer


async def test_review_asked_to_send_without_a_key_fails_and_sends_nothing(server, model):
    answer = await server.call_tool("arclang_review", {"model_path": str(model), "send": True})
    assert "FAILED" in answer.splitlines()[0]
    assert "TYPESAFE_API_KEY" in answer


async def test_trace_analysis_reports_coverage(server, model):
    answer = await server.call_tool(
        "arclang_trace_analysis", {"model_path": str(model), "matrix": True}
    )
    assert "coverage 100.0%" in answer.splitlines()[0]
    assert "Computes and commands the braking force" in answer


async def test_safety_check_is_not_presented_as_a_verdict(server, model):
    answer = await server.call_tool(
        "arclang_safety_check", {"model_path": str(model), "standard": "iso26262"}
    )
    assert "not a compliance verdict" in answer.splitlines()[0]
    assert "Hazards: 1" in answer


async def test_hazard_analysis_lists_declared_hazards_only(server, model):
    answer = await server.call_tool("arclang_hazard_analysis", {"model_path": str(model)})
    assert "1 hazard(s)" in answer.splitlines()[0]
    assert "H-001" in answer


async def test_merge_says_it_is_not_available(server):
    answer = await server.call_tool(
        "arclang_git_merge", {"base_path": "a", "ours_path": "b", "theirs_path": "c"}
    )
    assert "not available" in answer


async def test_viewpoint_diagrams_are_written(server, model, workspace):
    answer = await server.call_tool(
        "arclang_viewpoint_diagrams",
        {
            "model_path": str(model),
            "format": "viewpoints",
            "view": "lab",
            "output_path": "lab.json",
        },
    )
    assert "succeeded" in answer
    assert json.loads((workspace / "lab.json").read_text())


async def test_all_diagrams_reports_what_was_not_generated(server, model, workspace):
    answer = await server.call_tool(
        "arclang_generate_all_diagrams", {"model_path": str(model), "output_dir": "diagrams"}
    )
    written = sorted(path.stem for path in (workspace / "diagrams").glob("*.svg"))
    assert f"{len(written)}/{len(DIAGRAM_TYPES)} generated" in answer
    assert "component" in written
    for diagram_type in set(DIAGRAM_TYPES) - set(written):
        assert f"- {diagram_type}: NOT generated" in answer


async def test_a_binary_that_cannot_run_is_a_failure(workspace):
    compiler = ArcLangCompiler({})
    compiler.binary_path = str(workspace / "no-such-binary")
    run = await compiler.version()
    assert not run.ok and "cannot run" in run.stderr


async def test_an_older_compiler_is_called_out_in_every_answer(server, workspace):
    old = workspace / "old-arclang"
    old.write_text('#!/bin/sh\necho "arclang 1.0.0"\n')
    old.chmod(0o755)
    server.compiler.binary_path = str(old)
    answer = await server.call_tool("arclang_metamodel", {})
    assert "WARNING: the compiler at" in answer and "'arclang 1.0.0'" in answer


async def test_the_current_compiler_gets_no_warning(server, model):
    answer = await server.call_tool("arclang_info", {"model_path": str(model)})
    assert "WARNING" not in answer


async def test_validate_flags_warnings_that_block_the_gate(server, model):
    model.write_text(model.read_text().replace('priority: "Critical"', 'priority: "Urgent"'))
    first = (await server.call_tool("arclang_validate", {"model_path": str(model)})).splitlines()[0]
    assert "1 compilation warning(s)" in first
    assert "blockers of the gate" in first
    assert "valid" not in first


async def test_coverage_is_the_compilers_line_not_a_rationale(server, model):
    model.write_text(
        model.read_text().replace(
            'rationale: "Detects the imminent collision"',
            'rationale: "Traceability to ISO 26262 is covered at 12% of the effort"',
        )
    )
    answer = await server.call_tool(
        "arclang_trace_analysis", {"model_path": str(model), "matrix": True}
    )
    assert "12% of the effort" in answer
    assert "coverage 100.0%" in answer.splitlines()[0]


async def test_a_gate_verdict_needs_the_exit_code_too(server, workspace, model):
    named = workspace / "PRODUCTION GATE: PASS.arc"
    named.write_text(model.read_text().replace('verifies: ["REQ-BRK-001"]', 'verifies: ["REQ-X"]'))
    first = (await server.call_tool("arclang_gate", {"model_path": str(named)})).splitlines()[0]
    assert first.endswith("FAIL, 3 blocker(s)")


async def test_review_asked_to_send_with_nothing_to_review_claims_no_advice(
    server, workspace, monkeypatch
):
    monkeypatch.setenv("TYPESAFE_API_KEY", "not-a-key-and-never-used")
    empty = workspace / "empty.arc"
    empty.write_text('model M {\n  architecture logical { component "A" { id: "A1" } }\n}\n')
    answer = await server.call_tool("arclang_review", {"model_path": str(empty), "send": True})
    assert "advice received" not in answer
    assert "nothing to review" in answer.splitlines()[0]


@pytest.mark.parametrize(
    "tool, arguments",
    [
        ("arclang_review", {"model_path": "braking.arc", "send": "false"}),
        (
            "arclang_set_attribute",
            {
                "model_path": "braking.arc",
                "element": "LC-002",
                "key": "k",
                "value": '"v"',
                "write": "no",
            },
        ),
        ("arclang_format", {"path": "braking.arc", "write": 1}),
        ("arclang_trace_analysis", {"model_path": "braking.arc", "matrix": "yes"}),
        ("arclang_review", {"model_path": "braking.arc", "max_questions": "5"}),
        ("arclang_gate", {"model_path": "braking.arc", "standard": "--help"}),
        ("arclang_validate", {}),
        ("arclang_validate", {"model_path": "braking.arc", "strict": True}),
    ],
)
async def test_arguments_of_the_wrong_type_are_refused_before_anything_runs(
    server, model, tool, arguments
):
    before = model.read_text()
    with pytest.raises(ValueError, match="invalid arguments"):
        await server.call_tool(tool, arguments)
    assert model.read_text() == before


def test_the_workspace_file_cannot_choose_the_program(workspace, binary):
    from arclang_mcp import ArcLangMCPServer

    (workspace / ".arclang-mcp.toml").write_text(
        '[compiler]\npath = "/usr/bin/touch"\ntimeout = 7\n'
    )
    server = ArcLangMCPServer(workspace)
    assert server.compiler.binary_path == binary
    assert server.compiler.timeout == 7
