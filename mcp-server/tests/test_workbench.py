"""The workbench read and the scripted edits are the compiler's workbench API, the one
the VS Code extension and its agent use: the same ids, the same refusals."""

import json
import shutil
from pathlib import Path

import pytest

from arclang_mcp.compiler.wrapper import EDIT_OPS, WORKBENCH_GATE_STANDARDS
from arclang_mcp.tools.common import InvalidArguments

REPOSITORY = Path(__file__).resolve().parents[2]
EXAMPLE = REPOSITORY / "examples" / "complete_emergency_braking_simple.arc"
HEADLINE = (
    "Workbench of aeb.arc: compiles; 140 element(s), 0 finding(s), "
    "gate PASS, 7/7 requirements verified"
)
ADD_AND_DESCRIBE = [
    {"op": "add", "keyword": "function", "name": "Log braking", "container": "LC-BrakeController"},
    {"op": "set", "element": "Log braking", "key": "description", "value": '"Logs each braking"'},
]


@pytest.fixture
def aeb(workspace: Path) -> Path:
    """A copy of the flagship example in the workspace."""
    target = workspace / "aeb.arc"
    shutil.copy(EXAMPLE, target)
    return target


def body_of(answer: str) -> str:
    return answer.partition("\n\n")[2]


async def test_workbench_counts_the_elements_and_names_the_diagrams(server, aeb):
    answer = await server.call_tool("arclang_workbench", {"path": str(aeb)})
    assert answer.splitlines()[0] == HEADLINE, answer
    counts = next(line for line in answer.splitlines() if line.startswith("Elements by kind:"))
    assert "LogicalComponent 10" in counts and "Requirement 7" in counts
    assert "140 elements: id | kind | name | owner | attributes" in answer
    assert (
        "LC-BrakeController | LogicalComponent | BrakeController | "
        "Automotive Emergency Braking System (AEB) | "
        'description="Emergency brake control" safety_level="ASIL-D" type="Controller"'
    ) in answer
    assert "Relationships: 96 (" in answer and "traces: 15." in answer
    assert "Findings: 0" in answer
    assert "Gate: PASS, 7/7 requirements verified." in answer
    assert "Diagrams: 10 (id | view | title)" in answer
    assert "  lab | lab | AEB System - Component Structure" in answer
    assert "  es:EmergencyStop | es | EmergencyStop" in answer


async def test_workbench_filters_by_kind(server, aeb):
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "kind": "logicalcomponent"}
    )
    assert "10 elements: id | kind | name | owner | attributes" in answer
    listed = [line for line in answer.splitlines() if " | LogicalComponent | " in line]
    assert len(listed) == 10
    assert " | SystemFunction | " not in answer
    assert "Elements by kind:" in answer, "the counts are those of the whole model"


async def test_workbench_filters_by_layer_from_the_metamodel(server, aeb):
    answer = await server.call_tool("arclang_workbench", {"path": str(aeb), "layer": "Physical"})
    assert "16 elements: id | kind | name | owner | attributes" in answer, answer
    assert " | PhysicalNode | " in answer and " | BehaviorComponent | " in answer
    assert " | LogicalComponent | " not in answer


async def test_workbench_of_one_element_lists_its_relationships_and_traces(server, aeb):
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "element": "LC-BrakeController"}
    )
    assert "1 element: id | kind | name | owner | attributes" in answer
    assert "Incoming: Message critical threat from LC-ThreatAssessment" in answer
    outgoing = next(line for line in answer.splitlines() if line.startswith("Outgoing:"))
    assert "Deployment to PN-CentralADASECU" in outgoing
    assert "satisfies to REQ-AEB-004" in outgoing
    assert "Relationships:" not in answer
    by_name = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "element": "BrakeController"}
    )
    assert "LC-BrakeController | LogicalComponent | BrakeController" in by_name


async def test_workbench_of_an_unknown_element_says_so(server, aeb):
    answer = await server.call_tool("arclang_workbench", {"path": str(aeb), "element": "NOPE"})
    assert "0 elements: id | kind | name | owner | attributes" in answer
    assert "No element has the id or name 'NOPE'." in answer


async def test_workbench_raw_section_is_the_compilers_json(server, aeb):
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "raw": True, "section": "gate"}
    )
    assert answer.splitlines()[0] == "Workbench of aeb.arc: the 'gate' section"
    assert json.loads(body_of(answer)) == {
        "passed": True,
        "requirements_total": 7,
        "requirements_verified": 7,
        "findings": [],
    }
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "raw": True, "kind": "LogicalComponent"}
    )
    elements = json.loads(body_of(answer))
    assert [e["kind"] for e in elements["elements"]] == ["LogicalComponent"] * 10
    uuids = {e["uuid"] for e in elements["elements"]}
    assert elements["relationships"], "the relationships with an end among them are kept"
    assert all(r["source"] in uuids or r["target"] in uuids for r in elements["relationships"])


async def test_workbench_renders_the_exports_only_when_asked(server, aeb):
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "raw": True, "section": "proto"}
    )
    assert answer.endswith("printed no 'proto' section (it is rendered only with exports: true)")
    answer = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "raw": True, "section": "proto", "exports": True}
    )
    assert "message " in body_of(answer)
    summary = await server.call_tool("arclang_workbench", {"path": str(aeb), "exports": True})
    assert "Exports rendered (sysmlv2 " in summary


async def test_workbench_of_a_model_that_does_not_compile(server, broken):
    answer = await server.call_tool("arclang_workbench", {"path": str(broken)})
    assert answer.splitlines()[0] == "Workbench of broken.arc: does not compile"
    assert body_of(answer).strip(), "the compiler's errors are relayed"


async def test_workbench_arguments_are_the_clis(server, aeb):
    with pytest.raises(InvalidArguments, match="'gate_standard' must be one of"):
        await server.call_tool("arclang_workbench", {"path": str(aeb), "gate_standard": "iso26262"})
    with pytest.raises(InvalidArguments, match="'section' must be one of"):
        await server.call_tool("arclang_workbench", {"path": str(aeb), "section": "all"})
    help_text = (await server.compiler.run(["workbench", "--help"])).stdout
    for standard in WORKBENCH_GATE_STANDARDS:
        assert standard in help_text
    help_text = (await server.compiler.run(["edit", "--help"])).stdout
    for op in EDIT_OPS:
        assert f'"{op}"' in help_text


async def test_edit_applies_the_requests_in_order_without_writing(server, aeb):
    before = aeb.read_text()
    answer = await server.call_tool(
        "arclang_edit", {"path": str(aeb), "requests": ADD_AND_DESCRIBE}
    )
    assert answer.splitlines()[0] == (
        "Edit of aeb.arc (2 requests): accepted, NOT written (pass write: true to apply)"
    ), answer
    assert '+        function "Log braking" { description: "Logs each braking" }' in answer
    assert 'Applied in order:\n1. add function "Log braking" inside LC-BrakeController\n' in answer
    assert '2. set `description: "Logs each braking"` on Log braking' in answer
    assert aeb.read_text() == before


async def test_edit_writes_when_asked_and_the_model_still_passes(server, aeb):
    answer = await server.call_tool(
        "arclang_edit", {"path": str(aeb), "requests": ADD_AND_DESCRIBE, "write": True}
    )
    assert answer.splitlines()[0] == "Edit of aeb.arc (2 requests): written to aeb.arc", answer
    text = aeb.read_text()
    assert 'function "Log braking" { description: "Logs each braking" }' in text
    assert "// Automotive Emergency Braking System (AEB)" in text, "comments are kept"
    assert (await server.compiler.gate(aeb)).ok
    listed = await server.call_tool(
        "arclang_workbench", {"path": str(aeb), "element": "Log braking"}
    )
    assert (
        "Log braking | LogicalFunction | Log braking | LC-BrakeController | "
        'description="Logs each braking"'
    ) in listed


async def test_a_refused_request_leaves_the_file_untouched_and_is_named(server, aeb):
    before = aeb.read_text()
    answer = await server.call_tool(
        "arclang_edit",
        {
            "path": str(aeb),
            "requests": [
                {
                    "op": "set",
                    "element": "LC-BrakeController",
                    "key": "description",
                    "value": '"x"',
                },
                {"op": "set", "element": "NOPE", "key": "description", "value": '"x"'},
            ],
            "write": True,
        },
    )
    assert answer.splitlines()[0] == (
        'Edit of aeb.arc (2 requests): refused at request 2 (set `description: "x"` on NOPE): '
        "no element with id or name 'NOPE' is declared in this file"
    ), answer
    assert answer.endswith("Nothing was changed.")
    assert aeb.read_text() == before


async def test_edit_accepts_the_ids_the_workbench_lists_even_when_derived(server, aeb):
    """`SF-AssessThreat` is derived from the name: the text writes no id for it."""
    assert 'id: "SF-AssessThreat"' not in aeb.read_text()
    answer = await server.call_tool(
        "arclang_edit",
        {
            "path": str(aeb),
            "requests": [
                {"op": "set", "element": "SF-AssessThreat", "key": "latency", "value": "5 ms"}
            ],
        },
    )
    assert "NOT written" in answer.splitlines()[0], answer
    assert "+" in answer and "latency: 5 ms" in answer


async def test_edit_with_the_value_already_there_is_not_called_a_write(server, aeb):
    before = aeb.read_text()
    answer = await server.call_tool(
        "arclang_edit",
        {
            "path": str(aeb),
            "requests": [
                {
                    "op": "set",
                    "element": "LC-BrakeController",
                    "key": "description",
                    "value": '"Emergency brake control"',
                }
            ],
            "write": True,
        },
    )
    assert answer.splitlines()[0] == "Edit of aeb.arc (1 request): no change, nothing written"
    assert aeb.read_text() == before


async def test_edit_arguments_are_checked_to_the_leaves(server, aeb):
    before = aeb.read_text()
    with pytest.raises(InvalidArguments, match=r"\.op must be one of"):
        await server.call_tool("arclang_edit", {"path": str(aeb), "requests": [{"op": "fly"}]})
    with pytest.raises(InvalidArguments, match="is not an argument of"):
        await server.call_tool(
            "arclang_edit",
            {"path": str(aeb), "requests": [{"op": "remove", "element": "X", "colour": "red"}]},
        )
    with pytest.raises(InvalidArguments, match="'write' must be a boolean"):
        await server.call_tool(
            "arclang_edit",
            {"path": str(aeb), "requests": [{"op": "remove", "element": "X"}], "write": "yes"},
        )
    with pytest.raises(InvalidArguments, match="at least one request"):
        await server.call_tool("arclang_edit", {"path": str(aeb), "requests": []})
    assert aeb.read_text() == before
