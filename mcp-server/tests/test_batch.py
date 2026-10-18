"""The batch tool reduces real results and reports failures as failures."""

import json

import pytest

from arclang_mcp.tools.common import InvalidArguments


async def run_batch(server, operations, **options):
    return json.loads(
        await server.call_tool("arclang_execute_batch", {"operations": operations, **options})
    )


async def test_batch_reads_figures_from_the_compiler(server, model):
    path = str(model)
    answer = await run_batch(
        server,
        [
            {"action": "compile_and_validate", "params": {"model_path": path}},
            {"action": "gate", "params": {"model_path": path}},
            {"action": "extract_metrics", "params": {"model_path": path}},
            {"action": "analyze_traceability", "params": {"model_path": path}},
            {"action": "safety_check", "params": {"model_path": path}},
            {
                "action": "export_filtered",
                "params": {"model_path": path, "filter_type": "components_only"},
            },
        ],
    )
    assert answer["summary"] == {"total_operations": 6, "executed": 6, "successful": 6, "failed": 0}
    results = [entry["result"] for entry in answer["results"]]
    assert results[0]["compiles"] is True and results[0]["metrics"]["requirements"] == 1
    assert results[1] == {"verdict": "PASS", "blockers": []}
    assert results[2]["traces"] == 3 and results[2]["traceability_coverage_percent"] == 100.0
    assert results[3]["traceability_coverage_percent"] == 100.0
    assert results[4]["hazards"] == 1
    assert results[5]["layer"] == "logical architecture"
    assert results[5]["logical_component_count"] == 2


async def test_a_model_that_does_not_compile_is_an_error(server, broken, model):
    answer = await run_batch(
        server,
        [
            {"action": "compile_and_validate", "params": {"model_path": str(broken)}},
            {"action": "extract_metrics", "params": {"model_path": str(model)}},
        ],
        stop_on_error=True,
    )
    assert answer["summary"]["failed"] == 1 and answer["summary"]["executed"] == 1
    assert "line 3" in answer["results"][0]["error"]


async def test_gate_failure_is_a_result_with_blockers(server, model):
    model.write_text(model.read_text().replace('mitigated_by: ["REQ-BRK-001"]', ""))
    answer = await run_batch(server, [{"action": "gate", "params": {"model_path": str(model)}}])
    result = answer["results"][0]["result"]
    assert result["verdict"] == "FAIL" and result["blocker_count"] == 1
    assert "safety.mitigation" in result["blockers"][0]


async def test_compare_models_is_the_semantic_diff(server, model, workspace):
    other = workspace / "other.arc"
    other.write_text(
        model.read_text().replace(
            'trace "LC-001" satisfies "REQ-BRK-001"', '// trace "LC-001" satisfies "REQ-BRK-001"'
        )
    )
    answer = await run_batch(
        server,
        [
            {
                "action": "compare_models",
                "params": {"model1_path": str(model), "model2_path": str(other)},
            }
        ],
    )
    assert answer["results"][0]["result"]["counts"]["traces_removed"] == 1


async def test_missing_parameter_is_an_error_of_that_operation(server):
    answer = await run_batch(server, [{"action": "gate", "params": {}}])
    assert "missing parameter 'model_path'" in answer["results"][0]["error"]


async def test_an_unknown_action_is_refused_before_anything_runs(server):
    with pytest.raises(InvalidArguments, match="teleport"):
        await run_batch(server, [{"action": "teleport", "params": {}}])


@pytest.mark.parametrize(
    "arguments, mention",
    [
        ({"operations": ["x"]}, "'operations'[0] must be an object"),
        ({"operations": [{"action": "gate", "params": "nope"}]}, "params must be an object"),
        ({"operations": [{"action": "gate", "params": {}, "filter": None}]}, "filter must be an object"),
        ({"operations": [{"params": {}}]}, "'action' is required"),
    ],
)
async def test_a_malformed_operation_is_refused_with_its_place(server, arguments, mention):
    with pytest.raises(InvalidArguments) as refusal:
        await server.call_tool("arclang_execute_batch", arguments)
    assert mention in str(refusal.value)


async def test_an_operation_that_raises_does_not_lose_the_others(server, model):
    async def broken(params, limit):
        raise RuntimeError("unexpected")

    server.batch_tools._actions["extract_metrics"] = broken
    answer = await run_batch(
        server,
        [
            {"action": "gate", "params": {"model_path": str(model)}},
            {"action": "extract_metrics", "params": {"model_path": str(model)}},
            {"action": "gate", "params": {"model_path": str(model)}},
        ],
    )
    statuses = [entry["status"] for entry in answer["results"]]
    assert statuses == ["success", "error", "success"], answer["results"]
    assert answer["results"][1]["error"] == "unexpected"


async def test_generating_no_diagram_is_an_error_not_a_success(server, workspace):
    answer = await run_batch(
        server,
        [{"action": "generate_all_diagrams", "params": {"model_path": "missing.arc"}}],
    )
    assert answer["results"][0]["status"] == "error"
    assert "no diagram was generated" in answer["results"][0]["error"]
    assert not (workspace / "diagrams").exists() or not any((workspace / "diagrams").iterdir())


async def test_enhance_diagram_names_what_it_cannot_do(server, workspace):
    svg = workspace / "d.svg"
    svg.write_text('<svg viewBox="0 0 100 50"><text font-size="10">a</text></svg>')
    answer = await run_batch(
        server,
        [
            {
                "action": "enhance_diagram",
                "params": {
                    "diagram_path": str(svg),
                    "enhancements": ["enlarge_text", "add_safety_borders"],
                },
            }
        ],
    )
    result = answer["results"][0]["result"]
    assert result["enhancements_applied"] == ["enlarge_text"]
    assert result["enhancements_not_implemented"] == ["add_safety_borders"]
    assert 'font-size="12"' in (workspace / "d_enhanced.svg").read_text()


async def test_an_enhancement_that_changes_nothing_is_not_listed_as_applied(server, workspace):
    svg = workspace / "plain.svg"
    svg.write_text('<svg><rect width="5"/></svg>')
    answer = await run_batch(
        server,
        [
            {
                "action": "enhance_diagram",
                "params": {"diagram_path": str(svg), "enhancements": ["enlarge_text"]},
            }
        ],
    )
    result = answer["results"][0]["result"]
    assert result["enhancements_applied"] == []
    assert result["enhancements_without_effect"] == ["enlarge_text"]
    assert result["enhanced"] is None
    assert not (workspace / "plain_enhanced.svg").exists(), "nothing changed, nothing written"
