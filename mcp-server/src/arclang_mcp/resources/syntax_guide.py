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

RULES = """# ArcLang syntax rules (language 4.0.0)

The compiler is STRICT: any construct outside the grammar is a compile error
with its line and column. Nothing is skipped silently. Do not guess: check
with `arclang_validate` after every change, and read the `arclang://metamodel`
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
   `epbs <name> { }`.
5. Give every element you will trace or reference an `id: "..."`. The id is
   its stable identity: renaming an element with an id changes nothing else.
6. Traces are top-level: `trace "LC-001" satisfies "REQ-001" { rationale: "..." }`.
   Relations: satisfies, implements, validates, realizes, refines. A trace to
   an unknown element is an error. Verification is not a trace: a `test_case`
   lists the requirements it covers in `verifies: [...]`.
7. Enumerations are spelled as the metamodel spells them: safety levels are
   `"ASIL-D"`, `"DAL-A"`, `"SIL-2"`, `"QM"`; priorities are `"Critical"`,
   `"High"`, `"Medium"`, `"Low"`. A value outside the enumeration is a
   `metamodel:` warning and a gate blocker.
8. Quantities are a number and a unit of the unit table: `25 ms`, `100 Mbps`,
   `512 MB`, `40 %`. An unknown unit is an error; a quantity of the wrong
   dimension for a typed attribute (`power: 3 ms`) is a `metamodel:` warning
   and a gate blocker.
9. Function ports are `port in <name>` or `port out <name>`; component ports
   may also be `inout`; physical node ports have no direction.
10. Multiplicity (logical components and physical nodes): `multiplicity: 4`,
    or `"0..1"`, `"1..*"`, `"*"`.
11. The production gate (`arclang_gate`) needs: every requirement satisfied by
    a trace and verified by a `test_case` (`verifies: ["REQ-001"]`), and, when
    safety levels are declared, a `safety_analysis` with `hazard` entries that
    are `mitigated_by` requirements.

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
