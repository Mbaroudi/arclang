"""
Workbench tools: the compiler's workbench API, the one the VS Code extension
and its agent use.

`arclang workbench` prints everything a modelling workbench shows of one
model (the elements with their ids, the diagrams, the findings, the gate):
`arclang_workbench` reduces it to what an assistant needs, the way the
extension's `arclang_model` tool does (editors/vscode/src/ai/tools.ts,
`describeModel`), or hands back one section as the compiler printed it.
`arclang edit` applies one request written as JSON: `arclang_edit` applies a
list of them in order on one text, refuses the whole list when the compiler
refuses one, and writes the file only when asked.
"""

import json
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Awaitable, Callable, Dict, List, Optional, Tuple

from mcp.types import Tool

from ..compiler.wrapper import (
    EDIT_OPS,
    WORKBENCH_GATE_STANDARDS,
    WORKBENCH_SECTIONS,
    ArcLangCompiler,
    parse_edit,
    parse_workbench,
)
from .common import InvalidArguments, diff_against, report, resolve_path, schema

# As many elements as the extension's agent reads: more would drown the answer.
UNFILTERED_LIMIT = 150
FILTERED_LIMIT = 400
FINDINGS_SHOWN = 20
# The sections `arclang workbench` fills only with `--exports`.
EXPORT_SECTIONS = ("sysmlv2", "reqif", "c_headers", "proto")
# The layers `arclang metamodel` assigns to element kinds.
LAYERS = ("Operational", "System", "Logical", "Physical", "Epbs", "Data", "Transverse")
WRITE = {
    "type": "boolean",
    "description": "Rewrite the file in place. Default: change nothing and return the diff",
    "default": False,
}
PATH = {"type": "string", "description": "Path to the .arc model file"}

# One edit request, as `arclang edit --request` takes it (EditRequest in
# src/compiler/workbench.rs); the field documentation is the VS Code
# extension's (editors/vscode/package.json, arclang_edit).
EDIT_REQUEST: Dict[str, Any] = {
    "type": "object",
    "description": (
        "One edit of the model text. op is one of set, unset, rename, add, add_raw, remove, "
        "move, add_trace, remove_trace, wrap_fragment. Elements are designated by the id "
        "arclang_workbench lists (or the name when the text writes none)."
    ),
    "properties": {
        "op": {"type": "string", "enum": list(EDIT_OPS)},
        "element": {"type": "string", "description": "set/unset/rename/remove/move: the element."},
        "key": {"type": "string", "description": "set/unset: the attribute."},
        "value": {
            "type": "string",
            "description": "set: the value as ArcLang source: '\"text\"', '25 ms', '[A, B]', "
            "'ASIL-D'.",
        },
        "name": {
            "type": "string",
            "description": "rename: the new name; add: the name of the new element.",
        },
        "keyword": {
            "type": "string",
            "description": "add: the declaration keyword: actor, entity, activity, function, "
            "component, node, behavior_component, 'port in', 'port out', interface, class, "
            "mission, capability, functional_chain, requirement...",
        },
        "attributes": {
            "type": "array",
            "items": {"type": "array", "items": {"type": "string"}, "minItems": 2, "maxItems": 2},
            "description": "add: [key, value] pairs written with the new element, values as "
            "ArcLang source.",
        },
        "declaration": {
            "type": "string",
            "description": "add_raw: one declaration as ArcLang source, e.g. 'component_exchange "
            '"Flow" { from_port: "LC-A.p" to_port: "LC-B.q" }\' or '
            '\'message "A" -> "B" "text"\'.',
        },
        "container": {
            "type": "string",
            "description": "add/add_raw/move: the element the declaration goes into.",
        },
        "layer": {
            "type": "string",
            "description": "add/add_raw/move: else the layer block: operational_analysis, "
            "system_analysis, logical_architecture, physical_architecture, epbs.",
        },
        "layer_name": {
            "type": "string",
            "description": "add/add_raw/move: the layer's name when several layers of that "
            "kind exist.",
        },
        "top_level": {
            "type": "boolean",
            "description": "add/add_raw: else at the end of the file (classes, data types, "
            "traces).",
        },
        "from": {"type": "string", "description": "add_trace/remove_trace: the source element."},
        "kind": {
            "type": "string",
            "description": "add_trace/remove_trace: satisfies, implements, realizes, allocates, "
            "verifies...; wrap_fragment: loop, opt, alt, par, seq, critical.",
        },
        "to": {"type": "string", "description": "add_trace/remove_trace: the target element."},
        "scenario": {"type": "string", "description": "wrap_fragment: the scenario."},
        "label": {"type": "string", "description": "wrap_fragment: the fragment's label."},
        "first": {
            "type": "string",
            "description": "wrap_fragment: the label of the first message wrapped.",
        },
        "last": {
            "type": "string",
            "description": "wrap_fragment: the label of the last message wrapped.",
        },
        "operands": {
            "type": "array",
            "items": {"type": "array", "items": {"type": "string"}},
            "description": "wrap_fragment: [message label, guard] pairs that cut the fragment "
            "into operands.",
        },
    },
    "required": ["op"],
}

TOOLS: List[Tool] = [
    Tool(
        name="arclang_workbench",
        description=(
            "Read a model through the compiler's workbench (`arclang workbench`), as the VS Code "
            "extension's agent does: counts by kind, the elements (id, kind, name, owner id, "
            "attributes) filtered by `kind`, `layer` or one `element` (then with its "
            "relationships and traces), the findings (rule, severity, element, message), the "
            "gate verdict and the diagrams (id, view, title). Use it before editing to know the "
            "exact ids: the ids it lists are accepted by arclang_edit even when the text writes "
            "none. With `raw: true` the answer is one `section` of the compiler's JSON instead "
            "(elements, diagrams, findings, gate, document; sysmlv2, reqif, c_headers, proto "
            "with `exports: true`), the elements section filtered the same way."
        ),
        inputSchema=schema(
            {
                "path": PATH,
                "gate_standard": {
                    "type": "string",
                    "enum": list(WORKBENCH_GATE_STANDARDS),
                    "description": "The standard the gate is run against",
                    "default": "ISO26262",
                },
                "exports": {
                    "type": "boolean",
                    "description": "Also render the interop exports (SysML v2, ReqIF, C headers, "
                    "Protobuf), read with `raw: true` and `section`",
                    "default": False,
                },
                "kind": {
                    "type": "string",
                    "description": "Only the elements of this kind (LogicalComponent, "
                    "SystemFunction, Requirement...)",
                },
                "layer": {
                    "type": "string",
                    "description": "Only the elements of this layer: " + ", ".join(LAYERS),
                },
                "element": {
                    "type": "string",
                    "description": "Only this element (id or name), with what it is connected to",
                },
                "raw": {
                    "type": "boolean",
                    "description": "Return one `section` of the compiler's JSON instead of the "
                    "summary",
                    "default": False,
                },
                "section": {
                    "type": "string",
                    "enum": list(WORKBENCH_SECTIONS),
                    "description": "With `raw: true`: the section returned (default elements)",
                    "default": "elements",
                },
            },
            "path",
        ),
    ),
    Tool(
        name="arclang_edit",
        description=(
            "Apply a list of edits to a .arc model through the compiler's source editor "
            "(`arclang edit`), in order on one text: add an element inside another, in a "
            "layer or at the top level (ports with keyword 'port in'/'port out'), add a raw "
            "declaration (exchanges, messages, transitions), remove, move, rename (identity "
            "kept), set/unset an attribute, add/remove a trace, wrap messages in a fragment. "
            "Comments and layout are kept. The whole list is refused, and nothing changes, if "
            "any edit would leave a model that does not compile or asks for an unknown "
            "element; the answer says which request and why. Elements are designated by the "
            "ids arclang_workbench lists. Without `write: true` nothing is written: the answer "
            "is the diff of the edited source. Prefer this over rewriting the file."
        ),
        inputSchema=schema(
            {
                "path": PATH,
                "requests": {
                    "type": "array",
                    "items": EDIT_REQUEST,
                    "minItems": 1,
                    "description": "The edits, applied in order on one text",
                },
                "write": WRITE,
            },
            "path",
            "requests",
        ),
    ),
]


@dataclass(frozen=True)
class Filter:
    """Which elements an answer lists: all, one kind, one layer, or one element."""

    kind: Optional[str] = None
    layer: Optional[str] = None
    element: Optional[str] = None

    @property
    def is_set(self) -> bool:
        return bool(self.kind or self.layer or self.element)

    def keeps(self, element: Dict[str, Any], layers: Dict[str, str]) -> bool:
        if self.element and self.element not in (element.get("id"), element.get("name")):
            return False
        if self.kind and str(element.get("kind", "")).lower() != self.kind.lower():
            return False
        if self.layer:
            layer = layers.get(str(element.get("kind", "")), "Transverse")
            if layer.lower() != self.layer.lower():
                return False
        return True


class WorkbenchTools:
    """The workbench read and the scripted edits."""

    def __init__(self, compiler: ArcLangCompiler, workspace_root: Path):
        self.compiler = compiler
        self.workspace_root = workspace_root
        self._handlers: Dict[str, Callable[[Dict[str, Any]], Awaitable[str]]] = {
            "arclang_workbench": self._workbench,
            "arclang_edit": self._edit,
        }

    async def execute(self, tool_name: str, arguments: Dict[str, Any]) -> str:
        """Execute a workbench tool."""
        handler = self._handlers.get(tool_name)
        if handler is None:
            raise ValueError(f"Unknown workbench tool: {tool_name}")
        return await handler(arguments)

    async def _workbench(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["path"])
        title = f"Workbench of {model.name}"
        run = await self.compiler.workbench(
            model,
            gate_standard=args.get("gate_standard", "ISO26262"),
            exports=args.get("exports", False),
        )
        document = parse_workbench(run)
        if document is None:
            return report(title, run)
        if not document.get("success"):
            errors = "\n".join(str(error) for error in document.get("errors") or [])
            return f"{title}: does not compile\n\n{errors or run.output}"
        selection = Filter(args.get("kind"), args.get("layer"), args.get("element"))
        layers = await self._layers() if selection.layer else {}
        if args.get("raw", False):
            return section_answer(
                title, document, args.get("section", "elements"), selection, layers
            )
        return f"{title}: {headline(document)}\n\n{describe_model(document, layers, selection)}"

    async def _layers(self) -> Dict[str, str]:
        """The layer of each element kind, as the compiler's metamodel assigns it."""
        run = await self.compiler.metamodel(as_json=True)
        if not run.ok:
            raise RuntimeError(f"the compiler did not print its metamodel: {run.output}")
        metamodel = json.loads(run.stdout)
        return {
            str(kind.get("name")): str(kind.get("layer", "Transverse"))
            for kind in metamodel.get("kinds", [])
        }

    async def _edit(self, args: Dict[str, Any]) -> str:
        model = resolve_path(self.workspace_root, args["path"])
        requests: List[Dict[str, Any]] = args["requests"]
        if not requests:
            raise InvalidArguments("'requests' must hold at least one request")
        write = args.get("write", False)
        count = f"{len(requests)} request{'s' if len(requests) != 1 else ''}"
        title = f"Edit of {model.name} ({count})"
        try:
            before = model.read_text()
        except OSError as failure:
            return f"{title}: cannot read {model}: {failure}"
        edited, refusal = await self._apply(model.name, before, requests)
        if refusal is not None:
            return f"{title}: {refusal}\n\nNothing was changed."
        applied = "Applied in order:\n" + "\n".join(
            f"{index}. {describe_request(request)}" for index, request in enumerate(requests, 1)
        )
        if not write:
            return f"{diff_against(title, model.name, before, edited).rstrip()}\n\n{applied}"
        if edited == before:
            return f"{title}: no change, nothing written\n\n{applied}"
        model.write_text(edited)
        return f"{title}: written to {model.name}\n\n{applied}"

    async def _apply(
        self, name: str, source: str, requests: List[Dict[str, Any]]
    ) -> Tuple[str, Optional[str]]:
        """The text after every request, or the refusal of the first one the compiler refused.

        Each request runs on the result of the previous one, on a copy in a
        private temporary directory: the model file is never touched here.
        """
        text = source
        with tempfile.TemporaryDirectory(prefix="arclang-edit-") as directory:
            scratch = Path(directory) / name
            for index, request in enumerate(requests, 1):
                scratch.write_text(text)
                run = await self.compiler.edit(scratch, request)
                answer = parse_edit(run)
                where = f"refused at request {index} ({describe_request(request)})"
                if answer is None:
                    printed = run.output or "(the compiler printed nothing)"
                    return text, f"{where}: the compiler printed no answer\n\n{printed}"
                if not answer.get("success") or not isinstance(answer.get("source"), str):
                    return text, f"{where}: {answer.get('error') or 'no reason given'}"
                text = answer["source"]
        return text, None


def headline(document: Dict[str, Any]) -> str:
    """The outcome line of a workbench read: element count, findings, gate."""
    records = (document.get("elements") or {}).get("elements") or []
    findings = document.get("findings") or []
    parts = [f"compiles; {len(records)} element(s), {len(findings)} finding(s)"]
    gate = document.get("gate")
    if isinstance(gate, dict):
        parts.append(f"gate {gate_verdict(gate)}")
    return ", ".join(parts)


def gate_verdict(gate: Dict[str, Any]) -> str:
    verified = gate.get("requirements_verified", 0)
    total = gate.get("requirements_total", 0)
    return f"{'PASS' if gate.get('passed') else 'FAIL'}, {verified}/{total} requirements verified"


def describe_model(document: Dict[str, Any], layers: Dict[str, str], selection: Filter) -> str:
    """The model as an agent reads it: counts, the elements asked for, findings, gate, diagrams.

    The same picture the VS Code extension's `arclang_model` tool gives
    (`describeModel` in editors/vscode/src/ai/tools.ts).
    """
    records: List[Dict[str, Any]] = (document.get("elements") or {}).get("elements") or []
    relationships: List[Dict[str, Any]] = (document.get("elements") or {}).get(
        "relationships"
    ) or []
    diagrams = document.get("diagrams") or {}
    links: List[Dict[str, Any]] = diagrams.get("links") or []
    by_uuid = {element.get("uuid"): element for element in records}
    counts: Dict[str, int] = {}
    for element in records:
        counts[element.get("kind", "?")] = counts.get(element.get("kind", "?"), 0) + 1
    lines = ["Elements by kind: " + ", ".join(f"{k} {n}" for k, n in counts.items()) + "."]

    wanted = [element for element in records if selection.keeps(element, layers)]
    limit = FILTERED_LIMIT if selection.is_set else UNFILTERED_LIMIT
    shown = f" (first {limit})" if len(wanted) > limit else ""
    plural = "" if len(wanted) == 1 else "s"
    lines.append(f"{len(wanted)} element{plural}{shown}: id | kind | name | owner | attributes")
    for element in wanted[:limit]:
        lines.append(element_line(element, by_uuid))
    if selection.element and not wanted:
        lines.append(f"No element has the id or name '{selection.element}'.")
    if selection.element and len(wanted) == 1:
        incoming, outgoing = neighbours(wanted[0], records, relationships, links)
        lines.append("Incoming: " + ("; ".join(incoming) or "none"))
        lines.append("Outgoing: " + ("; ".join(outgoing) or "none"))
    if not selection.element:
        kinds = ", ".join(dict.fromkeys(str(r.get("kind")) for r in relationships))
        lines.append(f"Relationships: {len(relationships)} ({kinds}); traces: {len(links)}.")

    lines += finding_lines(document.get("findings") or [])
    gate = document.get("gate")
    if isinstance(gate, dict):
        lines.append(f"Gate: {gate_verdict(gate)}.")
        for blocker in gate.get("findings") or []:
            lines.append(
                f"  {blocker.get('check')} ({blocker.get('severity')}): {blocker.get('message')}"
            )
    graphs = diagrams.get("graphs") or []
    lines.append(f"Diagrams: {len(graphs)}" + (" (id | view | title)" if graphs else ""))
    for graph in graphs:
        arc = graph.get("arc") or {}
        lines.append(f"  {graph.get('id')} | {arc.get('view')} | {arc.get('title')}")
    exports = [name for name in EXPORT_SECTIONS if isinstance(document.get(name), str)]
    if exports:
        sizes = ", ".join(f"{name} {len(document[name])} chars" for name in exports)
        lines.append(f"Exports rendered ({sizes}): read one with raw: true and section.")
    return "\n".join(lines)


def element_line(element: Dict[str, Any], by_uuid: Dict[Any, Dict[str, Any]]) -> str:
    owner = by_uuid.get(element.get("owner"), {}).get("id", "") if element.get("owner") else ""
    attributes = " ".join(
        f"{key}={json.dumps(value, ensure_ascii=False)}"
        for key, value in (element.get("attributes") or {}).items()
        if key not in ("id", "name")
    )
    return f"{element.get('id')} | {element.get('kind')} | {element.get('name')} | {owner} | {attributes}"


def finding_lines(findings: List[Dict[str, Any]]) -> List[str]:
    shown = f" (first {FINDINGS_SHOWN})" if len(findings) > FINDINGS_SHOWN else ""
    lines = [f"Findings: {len(findings)}{shown}"]
    for finding in findings[:FINDINGS_SHOWN]:
        where = f" on {finding.get('element')}" if finding.get("element") else ""
        lines.append(
            f"  {finding.get('rule')} ({finding.get('severity')}){where}: {finding.get('message')}"
        )
    return lines


def neighbours(
    element: Dict[str, Any],
    records: List[Dict[str, Any]],
    relationships: List[Dict[str, Any]],
    links: List[Dict[str, Any]],
) -> Tuple[List[str], List[str]]:
    """What one element is connected to: the relationships and traces it takes part in.

    A trace names its ends as the text wrote them, by id or by name: both
    are looked up.
    """
    by_uuid = {record.get("uuid"): record for record in records}
    by_designator: Dict[Any, Dict[str, Any]] = {}
    for record in records:
        by_designator.setdefault(record.get("name"), record)
        by_designator.setdefault(record.get("id"), record)
    designators = {element.get("id"), element.get("name")}
    incoming: List[str] = []
    outgoing: List[str] = []

    def label(kind: Any, name: Any, other: Optional[Dict[str, Any]], preposition: str) -> str:
        named = f"{kind} {name}" if name else str(kind)
        return f"{named} {preposition} {other.get('id') if other else '?'}"

    for relationship in relationships:
        if relationship.get("source") == element.get("uuid"):
            other = by_uuid.get(relationship.get("target"))
            outgoing.append(label(relationship.get("kind"), relationship.get("name"), other, "to"))
        if relationship.get("target") == element.get("uuid"):
            other = by_uuid.get(relationship.get("source"))
            incoming.append(
                label(relationship.get("kind"), relationship.get("name"), other, "from")
            )
    for link in links:
        if link.get("source") in designators:
            outgoing.append(
                label(link.get("kind"), "", by_designator.get(link.get("target")), "to")
            )
        if link.get("target") in designators:
            incoming.append(
                label(link.get("kind"), "", by_designator.get(link.get("source")), "from")
            )
    return incoming, outgoing


def section_answer(
    title: str,
    document: Dict[str, Any],
    section: str,
    selection: Filter,
    layers: Dict[str, str],
) -> str:
    """One section of the workbench document, as the compiler printed it.

    The elements section honours the filter: the elements kept, and the
    relationships with an end among them.
    """
    content = document.get(section)
    if content is None:
        hint = " (it is rendered only with exports: true)" if section in EXPORT_SECTIONS else ""
        return f"{title}: the compiler printed no '{section}' section{hint}"
    if section == "elements" and selection.is_set and isinstance(content, dict):
        kept = [e for e in content.get("elements") or [] if selection.keeps(e, layers)]
        uuids = {e.get("uuid") for e in kept}
        content = {
            "elements": kept,
            "relationships": [
                r
                for r in content.get("relationships") or []
                if r.get("source") in uuids or r.get("target") in uuids
            ],
        }
    body = (
        content if isinstance(content, str) else json.dumps(content, indent=2, ensure_ascii=False)
    )
    return f"{title}: the '{section}' section\n\n{body}"


def describe_request(request: Dict[str, Any]) -> str:
    """One request in a sentence, for the list of what was applied and for refusals."""
    op = request.get("op")
    element = request.get("element")
    place = (
        f"inside {request['container']}"
        if request.get("container")
        else f"in {request['layer']}" if request.get("layer") else "at the top level"
    )
    if op == "set":
        return f"set `{request.get('key')}: {request.get('value')}` on {element}"
    if op == "unset":
        return f"remove `{request.get('key')}` from {element}"
    if op == "rename":
        return f"rename {element} to \"{request.get('name')}\""
    if op == "add":
        return f"add {request.get('keyword')} \"{request.get('name')}\" {place}"
    if op == "add_raw":
        first_line = str(request.get("declaration", "")).split("\n")[0]
        return f"add the declaration `{first_line}` {place}"
    if op == "remove":
        return f"delete {element}"
    if op == "move":
        return f"move {element} into {request.get('container') or request.get('layer') or '?'}"
    if op == "add_trace":
        return f"trace {request.get('from')} {request.get('kind')} {request.get('to')}"
    if op == "remove_trace":
        return f"remove the trace {request.get('from')} {request.get('kind')} {request.get('to')}"
    if op == "wrap_fragment":
        return (
            f"wrap the messages {request.get('first')}...{request.get('last')} of "
            f"{request.get('scenario')} in a {request.get('kind')} fragment"
        )
    return json.dumps(request)
