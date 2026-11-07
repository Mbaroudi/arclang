"""
What the server tells a client about the language.

Nothing here is a second description of ArcLang that could drift from the
compiler: the rules are the short list below, the example is
`reference_model.arc` (compiled and gated by the test suite), the grammar
is `spec/GRAMMAR.ebnf` when the server runs from the repository, and the
metamodel is asked from the binary.
"""

from pathlib import Path
from typing import Optional

RESOURCES = Path(__file__).parent
# mcp-server/src/arclang_mcp/resources -> the repository root.
REPOSITORY = RESOURCES.parents[3]

RULES = """# ArcLang syntax rules

For the compiler this server runs (`arclang` 4.3; the `arclang://metamodel`
resource prints the language version). The compiler is STRICT: any construct
outside the grammar is a compile error with its line and column. Nothing is
skipped silently. Do not guess: check with `arclang_validate` after every
change (`as_json: true` names the rule and the fix of each finding, and
`arclang_rules` says what a rule means), and read the `arclang://metamodel`
resource (or call `arclang_metamodel` with a `kind`) to know which attributes
an element kind accepts and with which type.

## Rules that are most often broken

1. Requirement ids are STRINGS, followed by the title:
   `req "REQ-001" "Title" { description: "..." }`. An id such as `REQ-001`
   without quotes is an error: `-` is not part of an identifier.
2. Names are identifiers (`Driver`, dotted `Vehicle.Monitor`) or strings
   (`"Brake controller"`). Anything with a space or a dash needs quotes.
3. Requirements live in `requirements <kind> { }` (kind: `stakeholder`,
   `system`, `safety` or any identifier) or in a `system_analysis` block.
4. Layers: `operational_analysis <name> { }`, `system_analysis <name> { }`,
   `architecture logical { }` (or `logical_architecture <name> { }`),
   `architecture physical { }` (or `physical_architecture <name> { }`),
   `epbs <name> { }`. `arclang_transition` declares on the next layer what
   realizes the elements of this one (oa-to-sa, sa-to-la, la-to-pa).
5. Give every element you will trace or reference an `id: "..."`. The id is
   its stable identity: renaming an element with an id changes nothing else.
   Ports take one too: `port out threat { id: "LP-001" data_type: "ThreatLevel" }`.
6. Traces are top-level: `trace "LC-001" satisfies "REQ-001" { rationale: "..." }`.
   Relations: satisfies, implements, validates, realizes, refines. A trace to
   an unknown element is an error. Verification is not a trace: a `test_case`
   lists the requirements it covers in `verifies: [...]`.
7. Enumerations are spelled as the metamodel spells them: safety levels are
   `"ASIL-D"`, `"DAL-A"`, `"SIL-2"`, `"QM"`; priorities are `"Critical"`,
   `"High"`, `"Medium"`, `"Low"`. A value outside the enumeration is a
   `metamodel.type` finding and a gate blocker.
8. Quantities are a number and a unit of the unit table, UNQUOTED:
   `latency: 25 ms`, `bandwidth: 2 Mbps`, `ram: 512 MB`, `load: 40 %`.
   A bare number (`latency: 25`) is a `metamodel.unit` warning; an unknown
   unit is an error; a quantity of the wrong dimension for a typed attribute
   (`power: 3 ms`) is a `metamodel.type` finding and a gate blocker.
9. Function ports are `port in <name>` or `port out <name>`; component ports
   may also be `inout`; physical node ports have no direction. Exchanges name
   the ports: `component_exchange X { from_port: "LC-001.threat" to_port: "LC-002.threat" }`,
   `functional_exchange X { from: "SF-001.out" to: "SF-002.in" }`. Prefer the
   named form: the arrow form `"SF-001.p" -> "SF-002.p"` takes the first port's
   name as its own id.
10. Multiplicity (logical components and physical nodes): `multiplicity: 4`,
    or `"0..1"`, `"1..*"`, `"*"`.
11. Reusable definitions are top-level `type "Name" { attributes }` blocks,
    `type "B" extends "A" { }` specializing one; `is: "Name"` on an element
    gives it the type's attributes. Writing one of them differently on the
    element is an error.
12. A `constraint "Name" { assert: <expression> }` compares the model's
    values: `"LF-001".latency + "LF-002".latency <= 50 ms`,
    `sum("FC-001", latency) <= "FC-001".latency_budget`. One that is false is
    a `constraint.violated` finding and a gate blocker.
13. Modes and states: `state_machine Name { initial: "A" mode A { } state B { }
    transition A -> B { trigger: "LF-001" guard: "..." } }`. A `mode` is a
    chosen behavior, a `state` an undergone one. The ends of a transition
    must be declared; a `trigger` that names no declared element is a warning.
14. Scenarios: `scenario Name { participants: ["LC-001", "LC-002"]
    message "LC-001" -> "LC-002" "label" { type: "async" exchange: "X" } }`.
    Participants must be declared elements, and every message end a participant.
15. Data model, top-level: `class Name { id: "..." field: "type" }`,
    `enumeration Name { values: [A, B] }`, `data_type Name { base: "float" unit: "N" }`,
    `exchange_item Name { mechanism: "EVENT" elements: ["EN-001"] }`
    (mechanism: EVENT, FLOW, OPERATION, DATA or SHARED_DATA). An exchange
    names what it carries with `exchange_item: "Name"`.
16. Physical architecture: a `node` `deploys "LC-001"`; a
    `link "Name" { from: "PN-001" to: "PN-002" }` joins two nodes; a
    `physical_path "Name" { involves: ["Link"] }` orders links by name.
17. The production gate (`arclang_gate`) needs: every requirement satisfied by
    a trace and verified by a `test_case` (`verifies: ["REQ-001"]`), every
    constraint true, no `metamodel.type` finding and, when safety levels are
    declared, a `safety_analysis` with `hazard` entries that are
    `mitigated_by` requirements (for `do178c`, each hazard also states its
    failure condition).

## Editing a model: precise requests, not a rewrite

Do not rewrite a file to change one thing: the compiler edits the text
itself and keeps comments, order and layout. Call `arclang_workbench` first
to learn the exact ids (it lists every element as `id | kind | name | owner |
attributes`, and one `element` with its relationships and traces; the ids it
lists are accepted even when the text writes none). Then call `arclang_edit`
with a list of requests, applied in order on one text:

    {"op": "add", "keyword": "function", "name": "Log braking", "container": "LC-002"}
    {"op": "set", "element": "Log braking", "key": "description", "value": "\"Logs each braking\""}
    {"op": "add_trace", "from": "LC-002", "kind": "satisfies", "to": "REQ-BRK-001"}

The other ops are `unset`, `rename` (the identity is kept), `add_raw` (one
declaration as source: an exchange, a message, a transition), `remove`,
`move` (a function reallocated, a component nested elsewhere),
`remove_trace` and `wrap_fragment`. The whole list is refused, and nothing
changes, when any request names an unknown element or would leave a model
that does not compile; the answer names the request and the reason. Nothing
is written without `write: true`: the answer is the diff.

## A model that compiles and passes the gate

```arc
{reference_model}
```
"""


def reference_model() -> str:
    """The reference model: every construct in it is accepted by the compiler."""
    return (RESOURCES / "reference_model.arc").read_text().rstrip()


def get_syntax_rules() -> str:
    """The syntax rules given to AI clients, with the reference model."""
    return RULES.replace("{reference_model}", reference_model())


def get_grammar() -> Optional[str]:
    """The canonical grammar, when the server runs from the ArcLang repository."""
    grammar = REPOSITORY / "spec" / "GRAMMAR.ebnf"
    return grammar.read_text() if grammar.is_file() else None


DOCUMENT_HEADER = """<!-- GENERATED from src/arclang_mcp/resources/syntax_guide.py and reference_model.arc.
     Do not edit: a test fails when this file and the server disagree.
     Regenerate with: python -m arclang_mcp.resources > ARCLANG_SYNTAX_RULES.md -->

"""


def rules_document() -> str:
    """The content of ARCLANG_SYNTAX_RULES.md: the rules the server serves."""
    return DOCUMENT_HEADER + get_syntax_rules()
