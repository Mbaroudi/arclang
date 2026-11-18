"""What the server tells a client about the language must be true of the compiler."""

import json
import re

from arclang_mcp.resources.syntax_guide import get_syntax_rules, reference_model
from arclang_mcp.server import GRAMMAR_URI, METAMODEL_URI, REFERENCE_MODEL_URI, SYNTAX_RULES_URI


async def test_reference_model_compiles_is_formatted_and_passes_the_gate(server, model):
    assert (await server.compiler.check(model)).ok
    assert (await server.compiler.format(model, check=True)).ok
    gate = await server.compiler.gate(model)
    assert "PRODUCTION GATE: PASS" in gate.stdout, gate.output


async def test_reference_model_has_no_typed_attribute_violation(server, model):
    check = await server.compiler.check(model)
    assert "metamodel:" not in check.stdout


def test_syntax_rules_embed_the_reference_model():
    assert reference_model() in get_syntax_rules()


async def test_one_line_examples_of_the_rules_compile(server, workspace):
    """Each `trace ...` and `req ...` example written in the rules is real syntax."""
    rules = get_syntax_rules().split("## A model that compiles")[0]
    requirement = re.search(r'`(req "REQ-001" [^`]+)`', rules).group(1)
    trace = re.search(r'`(trace "LC-001" [^`]+)`', rules).group(1)
    candidate = workspace / "rules.arc"
    candidate.write_text(
        f"model M {{\n requirements system {{ {requirement} }}\n"
        f' architecture logical {{ component "C" {{ id: "LC-001" multiplicity: "1..*" }} }}\n}}\n'
        f"{trace}\n"
    )
    check = await server.compiler.check(candidate)
    assert check.ok, check.output


async def test_every_trace_relation_named_in_the_rules_is_accepted(server, workspace):
    rules = get_syntax_rules()
    relations = re.search(r"Relations: ([a-z, ]+)\.", rules).group(1).split(", ")
    assert len(relations) >= 5
    for relation in relations:
        candidate = workspace / f"{relation}.arc"
        candidate.write_text(
            'model M {\n requirements system { req "R1" "t" { description: "d" } }\n'
            ' architecture logical { component "C" { id: "C1" } }\n}\n'
            f'trace "C1" {relation} "R1" {{ rationale: "r" }}\n'
        )
        check = await server.compiler.check(candidate)
        assert check.ok, f"{relation}: {check.output}"


async def test_resources_are_listed_and_readable(server):
    uris = {str(resource.uri) for resource in server.list_resources()}
    assert {SYNTAX_RULES_URI, REFERENCE_MODEL_URI, METAMODEL_URI, GRAMMAR_URI} <= uris
    for uri in uris:
        assert (await server.read_resource(uri)).strip()


async def test_metamodel_resource_comes_from_the_compiler(server):
    metamodel = await server.read_resource(METAMODEL_URI)
    generated = (await server.compiler.metamodel()).stdout
    assert metamodel.strip() == generated.strip()
    assert "Language version **" in metamodel
    assert "LogicalComponent" in metamodel


def test_the_rules_file_of_the_repository_is_the_one_the_server_serves():
    from pathlib import Path

    from arclang_mcp.resources.syntax_guide import rules_document

    written = Path(__file__).resolve().parents[1] / "ARCLANG_SYNTAX_RULES.md"
    assert written.read_text() == rules_document()


async def test_reference_model_has_no_finding_of_any_rule(server, model):
    check = await server.compiler.check(model, as_json=True)
    document = json.loads(check.stdout)
    assert document["success"] is True and document["findings"] == [], check.output


CONSTRUCTS_NAMED_IN_THE_RULES = (
    "type ",
    "extends ",
    "is: ",
    "constraint ",
    "assert: ",
    "state_machine ",
    "transition ",
    "scenario ",
    "message ",
    "class ",
    "enumeration ",
    "data_type ",
    "exchange_item ",
    "exchange_item: ",
    "physical_path ",
    "link ",
    "deploys ",
    "functional_exchange ",
    "component_exchange ",
    "port in ",
    "port out ",
    "test_case ",
    "safety_analysis",
    "mitigated_by: ",
    "verifies: ",
)


def test_every_construct_the_rules_teach_is_proven_by_the_reference_model():
    """A construct named in the rules is written in the model the tests compile and gate."""
    model = reference_model()
    for construct in CONSTRUCTS_NAMED_IN_THE_RULES:
        assert construct in model, construct


async def test_every_rule_id_named_in_the_rules_is_in_the_compilers_catalogue(server):
    rules = get_syntax_rules().split("## A model that compiles")[0]
    pattern = r"`((?:metamodel|constraint|trace|identity|syntax|method)\.[a-z-]+)`"
    named = set(re.findall(pattern, rules))
    assert named, "the rules name no rule id"
    printed = (await server.compiler.rules(as_json=True)).stdout
    catalog = {entry["id"] for entry in json.loads(printed)}
    assert named <= catalog, named - catalog


async def test_the_findings_the_rules_promise_are_the_ones_the_compiler_gives(server, workspace):
    candidate = workspace / "promised.arc"
    candidate.write_text(
        "model M {\n architecture logical {\n"
        '  component "C" { id: "C1" power: 3 ms '
        'function "F" { id: "F1" latency: 25 } }\n }\n}\n'
    )
    check = await server.compiler.check(candidate, as_json=True)
    rules = {finding["rule"] for finding in json.loads(check.stdout)["findings"]}
    assert {"metamodel.type", "metamodel.unit"} <= rules, check.output


async def test_a_quoted_requirement_id_and_an_unquoted_quantity_compile(server, workspace):
    candidate = workspace / "quantity.arc"
    candidate.write_text(
        "model M {\n architecture logical {\n"
        '  component "C" { id: "C1" multiplicity: "1..*" '
        'function "F" { id: "F1" latency: 25 ms } }\n }\n}\n'
    )
    check = await server.compiler.check(candidate, as_json=True)
    assert check.ok
    findings = json.loads(check.stdout)["findings"]
    assert not [f for f in findings if f["rule"].startswith("metamodel")]


# ------------------------------------------- the shorter examples of the rules


def fenced_models(rules: str):
    """Every fenced block of ArcLang in the rules: each is a model of its own."""
    return re.findall(r"^```arc\n(.*?)^```", rules, flags=re.M | re.S)


async def test_every_fenced_model_of_the_rules_compiles_and_is_formatted(server, workspace):
    from arclang_mcp.resources.syntax_guide import EXAMPLES, example

    blocks = fenced_models(get_syntax_rules())
    assert len(blocks) == len(EXAMPLES) + 1, "the examples and the reference model"
    for number, block in enumerate(blocks):
        candidate = workspace / f"block_{number}.arc"
        candidate.write_text(block)
        check = await server.compiler.check(candidate)
        assert check.ok and "No compilation errors" in check.stdout, f"block {number}: {check.output}"
        assert "metamodel:" not in check.stdout, f"block {number}: {check.stdout}"
        assert (await server.compiler.format(candidate, check=True)).ok, f"block {number} is not formatted"
    for name in EXAMPLES:
        assert example(name) + "\n" in blocks, name


def test_the_rules_teach_the_recent_constructs_in_a_model_that_compiles():
    """Each construct is named in the prose AND written in a fenced model (compiled above)."""
    rules = get_syntax_rules()
    written = "\n".join(fenced_models(rules))
    prose = re.sub(r"^```arc\n.*?^```", "", rules, flags=re.M | re.S)
    for construct in (
        "execute:", "simulate:", "expects:", "twin", "drives:", "compares:", "correspondence",
        "io_binding", "level:", "verification_levels:", "organization", "document",
        "responsible:", "documented_by:", "risk", "bears_on", "mitigated_by",
    ):
        assert construct in prose, f"the rules do not explain {construct}"
        assert re.search(rf"^\s*{re.escape(construct)}", written, flags=re.M), f"no model writes {construct}"


async def test_the_executed_case_is_one_the_compiler_can_run(server, workspace):
    from arclang_mcp.resources.syntax_guide import example

    model = workspace / "wiper.arc"
    model.write_text(example("executed_case"))
    listed = await server.call_tool("arclang_debug", {"model_path": "wiper.arc", "list": True})
    assert '"Wiper"' in listed
    ladder = await server.call_tool("arclang_realize", {"model_path": "wiper.arc"})
    # The case executes: the plan has the stages that run it, and names them.
    assert "arclang sil" in ladder and "TC-WIPE" in ladder
    schedule = await server.call_tool("arclang_schedule", {"model_path": "wiper.arc", "wcet": {"Wiper": 100}})
    assert "Wiper task" in schedule and schedule.splitlines()[0].endswith(": succeeded")
    # No verdict is recorded for it: until it is run, the gate does not pass on it.
    gate = await server.call_tool("arclang_gate", {"model_path": "wiper.arc"})
    assert gate.splitlines()[0].startswith("Production gate of wiper.arc: FAIL")


async def test_the_twin_of_the_rules_is_a_twin_for_the_compiler(server, workspace):
    from arclang_mcp.resources.syntax_guide import example

    (workspace / "heating.arc").write_text(example("twin"))
    twins = await server.call_tool("arclang_twin", {"model_path": "heating.arc", "list": True})
    assert json.loads(twins.split("\n\n", 1)[1])[0]["id"] == "TW-JANUARY"
    cases = await server.call_tool("arclang_verify", {"model_path": "heating.arc", "list": True})
    assert "TC-DESIGN-DAY" in cases


async def test_the_levels_of_the_v_block_the_gate_when_one_is_missing(server, workspace):
    from arclang_mcp.resources.syntax_guide import example

    model = workspace / "v.arc"
    model.write_text(example("v_model"))
    gate = await server.call_tool("arclang_gate", {"model_path": "v.arc"})
    assert gate.splitlines()[0] == "Production gate of v.arc: PASS", gate
    text = model.read_text()
    track = next(line for line in text.splitlines() if "TC-TRACK" in line)
    model.write_text(text.replace(track + "\n", ""))
    gate = await server.call_tool("arclang_gate", {"model_path": "v.arc"})
    assert gate.splitlines()[0].startswith("Production gate of v.arc: FAIL"), gate
    assert "validation level" in gate


async def test_responsibility_documents_and_risks_are_relationships(server, workspace):
    from arclang_mcp.resources.syntax_guide import example

    (workspace / "v.arc").write_text(example("v_model"))
    impact = await server.call_tool("arclang_impact", {"model_path": "v.arc", "element": "ORG-CHASSIS"})
    assert "LC-BRAKE" in impact and "REQ-STOP" in impact and "responsible for" in impact
    reached = await server.call_tool("arclang_impact", {"model_path": "v.arc", "element": "LC-BRAKE", "format": "json"})
    assert "RISK-1" in reached and "DOC-SRS" in await server.call_tool(
        "arclang_impact", {"model_path": "v.arc", "element": "REQ-STOP", "format": "json"}
    )


async def test_the_correspondence_of_the_rules_is_what_prove_reads(server, workspace, recorded):
    from arclang_mcp.resources.syntax_guide import example

    (workspace / "toggle.arc").write_text(example("correspondence"))
    await server.call_tool("arclang_prove", {"model_path": "toggle.arc", "correspondence": "Toggle sketch"})
    assert "--correspondence=Toggle sketch" in recorded.last


def test_the_example_of_the_correspondence_is_the_one_the_compiler_proves():
    """The same machine and correspondence as tests/fixtures/sketches/toggle.arc, which CI proves."""
    from pathlib import Path

    from arclang_mcp.resources.syntax_guide import example

    proven = (Path(__file__).resolve().parents[2] / "tests/fixtures/sketches/toggle.arc").read_text()
    squeezed = lambda text: re.sub(r"\s+", "", re.sub(r"^//.*$", "", text, flags=re.M))  # noqa: E731
    assert squeezed(example("correspondence")) == squeezed(proven)
