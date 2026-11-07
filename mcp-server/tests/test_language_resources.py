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
