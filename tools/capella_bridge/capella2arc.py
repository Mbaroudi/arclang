#!/usr/bin/env python3
"""Capella -> ArcLang bridge (M3 proof of concept).

Reads a real Capella model (.aird) with capellambse and emits an ArcLang
`.arc` file that compiles with the strict v3 parser.

Round-trip cornerstone: every emitted element carries its ORIGINAL Capella
UUID as the ArcLang `id:` attribute, so identity survives the conversion in
both directions.

Usage:
    python capella2arc.py path/to/model.aird [-o output.arc]
"""

from __future__ import annotations

import argparse
import re
import sys
import warnings

import capellambse


def esc(text: str | None) -> str:
    """Escape a value for an ArcLang double-quoted string.

    Preserves the text EXACTLY (including leading/trailing whitespace):
    round-trip fidelity requires byte-identical names.
    """
    if not text:
        return ""
    return (
        str(text)
        .replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\r", "\\r")
    )


def attr_block(indent: str, **attrs: str | None) -> list[str]:
    lines = []
    for key, value in attrs.items():
        if value:
            lines.append(f'{indent}{key}: "{esc(value)}"')
    return lines


def ref_list(indent: str, key: str, refs: list[str]) -> list[str]:
    """Render `key: ["ref", ...]` — an ArcLang reference list attribute."""
    if not refs:
        return []
    inner = ", ".join(f'"{esc(r)}"' for r in refs)
    return [f"{indent}{key}: [{inner}]"]


# Arcadia exchange mechanisms accepted by the strict parser.
MECHANISMS = {"EVENT", "FLOW", "OPERATION", "DATA", "SHARED_DATA"}


class ArcEmitter:
    def __init__(self, model: capellambse.MelodyModel):
        self.model = model
        self.lines: list[str] = []
        self.stats: dict[str, int] = {}
        self.emitted_components: set[str] = set()
        # Every element UUID emitted with an `id:` attribute AND registered as
        # an identity by the compiler — traces/involves/participants may only
        # reference UUIDs in this set (the strict parser rejects dangling refs).
        self.emitted_ids: set[str] = set()
        # Names of emitted elements (transition triggers resolve by name).
        self.emitted_names: set[str] = set()
        # Physical links are identified by NAME in ArcLang (no id field):
        # physical_path involves must reference names in this set.
        self.emitted_link_names: set[str] = set()
        self.emitted_requirements: set[str] = set()

    def track(self, uuid: str, name: str | None = None) -> None:
        self.emitted_ids.add(uuid)
        if name:
            self.emitted_names.add(name)

    def out(self, line: str = "") -> None:
        self.lines.append(line)

    def count(self, kind: str) -> None:
        self.stats[kind] = self.stats.get(kind, 0) + 1

    # ---- layers -----------------------------------------------------------

    def emit_model_header(self) -> None:
        name = esc(self.model.info.title) or "ImportedModel"
        self.out(f'model "{name}" {{')
        self.out(f'  description: "Imported from Capella by capella2arc"')
        capella_version = getattr(self.model.info, "capella_version", None)
        if capella_version:
            self.out(f'  capella_version: "{esc(capella_version)}"')
        self.out("}")
        self.out()

    def emit_operational_analysis(self) -> None:
        oa = self.model.oa
        actors = list(getattr(oa, "all_actors", []) or [])
        entities = [e for e in (getattr(oa, "all_entities", []) or []) if e not in actors]
        activities = list(getattr(oa, "all_activities", []) or [])
        if not (actors or entities or activities):
            return

        self.out('operational_analysis "Operational Analysis" {')
        for actor in actors:
            self.track(actor.uuid, actor.name)
            self.out(f'  actor "{esc(actor.name)}" {{')
            self.out(f'    id: "{actor.uuid}"')
            self.lines += attr_block("    ", description=getattr(actor, "description", None))
            self.out("  }")
            self.count("actor")
        for entity in entities:
            self.track(entity.uuid, entity.name)
            self.out(f'  entity "{esc(entity.name)}" {{')
            self.out(f'    id: "{entity.uuid}"')
            for activity in getattr(entity, "activities", []) or []:
                self.track(activity.uuid, activity.name)
                self.out(f'    activity "{esc(activity.name)}" {{')
                self.out(f'      id: "{activity.uuid}"')
                self.out("    }")
                self.count("activity")
            self.out("  }")
            self.count("entity")
        # Activities not owned by an emitted entity
        emitted = {
            a.uuid
            for e in entities
            for a in (getattr(e, "activities", []) or [])
        }
        for activity in activities:
            if activity.uuid in emitted:
                continue
            self.track(activity.uuid, activity.name)
            self.out(f'  operational_activity "{esc(activity.name)}" {{')
            self.out(f'    id: "{activity.uuid}"')
            self.out("  }")
            self.count("activity")

        # Communication means (Arcadia: CommunicationMean between entities).
        # from:/to: are REQUIRED by the parser — only emitted when both
        # endpoint entities/actors were emitted above.
        for mean in getattr(oa, "all_entity_exchanges", []) or []:
            source = getattr(mean, "source", None)
            target = getattr(mean, "target", None)
            src_uuid = getattr(source, "uuid", None)
            tgt_uuid = getattr(target, "uuid", None)
            if src_uuid not in self.emitted_ids or tgt_uuid not in self.emitted_ids:
                continue
            self.out(f'  communication_means "{esc(mean.name)}" {{')
            self.out(f'    id: "{mean.uuid}"')
            self.out(f'    from: "{src_uuid}"')
            self.out(f'    to: "{tgt_uuid}"')
            self.lines += attr_block("    ", description=getattr(mean, "description", None))
            self.out("  }")
            self.count("communication_means")

        # Operational processes (ordered involvements). The parser requires a
        # non-empty involves list — processes whose involvements are all
        # unemitted (interactions, ...) are skipped.
        # NOTE: not added to emitted_ids — the compiler does not register
        # operational processes as trace/involves targets.
        for process in getattr(oa, "all_processes", []) or []:
            involves = [
                item.uuid
                for item in (getattr(process, "involved", []) or [])
                if item.uuid in self.emitted_ids
            ]
            if not involves:
                continue
            self.out(f'  operational_process "{esc(process.name)}" {{')
            self.out(f'    id: "{process.uuid}"')
            self.lines += ref_list("    ", "involves", involves)
            self.lines += attr_block("    ", description=getattr(process, "description", None))
            self.out("  }")
            self.count("operational_process")
        self.out("}")
        self.out()

    # ---- Arcadia capabilities & chains (shared SA/LA helpers) --------------

    def emit_functional_chain(self, chain, indent: str) -> None:
        """Emit `functional_chain` — involves is REQUIRED non-empty, so a
        chain whose involvements were all unemitted is skipped entirely."""
        involves = [
            item.uuid
            for item in (getattr(chain, "involved", []) or [])
            if item.uuid in self.emitted_ids
        ]
        if not involves:
            self.count("functional_chain_skipped")
            return
        self.track(chain.uuid, chain.name)
        self.out(f'{indent}functional_chain "{esc(chain.name)}" {{')
        self.out(f'{indent}  id: "{chain.uuid}"')
        self.lines += ref_list(indent + "  ", "involves", involves)
        self.lines += attr_block(indent + "  ", description=getattr(chain, "description", None))
        self.out(f"{indent}}}")
        self.count("functional_chain")

    def emit_capability(self, capability, keyword: str, indent: str,
                        mission_of: dict[str, str] | None = None) -> None:
        """Emit `capability` (SA) or `capability_realization` (LA).

        All references (mission:, realizes:, involves:) are compile ERRORS
        when dangling — only UUIDs emitted in this file are referenced.
        """
        involves = []
        for attr in ("involved_functions", "involved_chains", "involved_components"):
            for item in getattr(capability, attr, []) or []:
                if item.uuid in self.emitted_ids and item.uuid not in involves:
                    involves.append(item.uuid)
        realizes = next(
            (
                realized.uuid
                for realized in (getattr(capability, "realized_capabilities", []) or [])
                if realized.uuid in self.emitted_ids
            ),
            None,
        )
        mission = (mission_of or {}).get(capability.uuid)
        self.track(capability.uuid, capability.name)
        self.out(f'{indent}{keyword} "{esc(capability.name)}" {{')
        self.out(f'{indent}  id: "{capability.uuid}"')
        if mission:
            self.out(f'{indent}  mission: "{mission}"')
        if realizes:
            self.out(f'{indent}  realizes: "{realizes}"')
        self.lines += ref_list(indent + "  ", "involves", involves)
        self.lines += attr_block(indent + "  ", description=getattr(capability, "description", None))
        self.out(f"{indent}}}")
        self.count(keyword)

    def emit_system_analysis(self) -> None:
        sa = self.model.sa
        functions = list(getattr(sa, "all_functions", []) or [])
        missions = list(getattr(sa, "all_missions", []) or [])
        capabilities = list(getattr(sa, "all_capabilities", []) or [])
        chains = list(getattr(sa, "all_functional_chains", []) or [])
        if not (functions or missions or capabilities or chains):
            return

        self.out('system_analysis "System Analysis" {')
        for function in functions:
            self.track(function.uuid, function.name)
            self.out(f'  function "{esc(function.name)}" {{')
            self.out(f'    id: "{function.uuid}"')
            self.lines += attr_block("    ", description=getattr(function, "description", None))
            self.out("  }")
            self.count("function")

        # Missions first (capabilities reference them via mission:), then
        # chains (capabilities may involve them), then capabilities.
        mission_of: dict[str, str] = {}
        for mission in missions:
            self.track(mission.uuid, mission.name)
            self.out(f'  mission "{esc(mission.name)}" {{')
            self.out(f'    id: "{mission.uuid}"')
            self.lines += attr_block("    ", description=getattr(mission, "description", None))
            self.out("  }")
            self.count("mission")
            # Capella models mission->capability via CapabilityExploitation.
            for exploited in getattr(mission, "exploits", []) or []:
                mission_of.setdefault(exploited.uuid, mission.uuid)
        for chain in chains:
            self.emit_functional_chain(chain, "  ")
        for capability in capabilities:
            self.emit_capability(capability, "capability", "  ", mission_of)
        self.out("}")
        self.out()

    def emit_component(self, component, indent: str) -> None:
        self.emitted_components.add(component.uuid)
        self.track(component.uuid, component.name)
        self.out(f'{indent}component "{esc(component.name)}" {{')
        self.out(f'{indent}  id: "{component.uuid}"')
        self.lines += attr_block(
            indent + "  ",
            description=getattr(component, "description", None),
        )
        for function in getattr(component, "allocated_functions", []) or []:
            self.track(function.uuid, function.name)
            self.out(f'{indent}  function "{esc(function.name)}" {{')
            self.out(f'{indent}    id: "{function.uuid}"')
            self.out(f"{indent}  }}")
            self.count("allocated_function")
        for child in getattr(component, "components", []) or []:
            self.emit_component(child, indent + "  ")
        self.out(f"{indent}}}")
        self.count("component")

    def emit_logical_architecture(self) -> None:
        la = self.model.la
        try:
            roots = list(la.component_package.components)
        except Exception:
            roots = list(getattr(la, "all_components", []) or [])
        if not roots:
            return

        self.out('logical_architecture "Logical Architecture" {')
        for component in roots:
            self.emit_component(component, "  ")

        # Exchange endpoints may live outside the root component package
        # (logical actors, ...) — emit any not-yet-emitted owner first so
        # every exchange endpoint resolves.
        exchanges = list(getattr(la, "all_component_exchanges", []) or [])
        endpoints = []
        for exchange in exchanges:
            source = getattr(exchange, "source", None)
            target = getattr(exchange, "target", None)
            src_owner = getattr(source, "owner", None) or source
            tgt_owner = getattr(target, "owner", None) or target
            if src_owner is None or tgt_owner is None:
                continue
            endpoints.append((exchange, src_owner, tgt_owner))
            for owner in (src_owner, tgt_owner):
                if owner.uuid not in self.emitted_components:
                    self.emit_component(owner, "  ")

        for exchange, src_owner, tgt_owner in endpoints:
            self.out(f'  component_exchange "{esc(exchange.name)}" {{')
            self.out(f'    from_port: "{src_owner.uuid}"')
            self.out(f'    to_port: "{tgt_owner.uuid}"')
            self.out("  }")
            self.count("component_exchange")

        # LA functional chains and capability realizations (Arcadia).
        for chain in getattr(la, "all_functional_chains", []) or []:
            self.emit_functional_chain(chain, "  ")
        for capability in getattr(la, "all_capabilities", []) or []:
            self.emit_capability(capability, "capability_realization", "  ")
        self.out("}")
        self.out()

    def emit_physical_architecture(self) -> None:
        pa = self.model.pa
        try:
            roots = list(pa.component_package.components)
        except Exception:
            roots = []
        # Physical link/path endpoints live on nested components and on
        # components in sub-packages — emit every PA component (flat: the
        # ArcLang PA grammar has no nested nodes) so link owners resolve.
        components = list(roots)
        seen = {c.uuid for c in roots}
        for component in getattr(pa, "all_components", []) or []:
            if component.uuid not in seen:
                seen.add(component.uuid)
                components.append(component)
        links = list(getattr(pa, "all_physical_links", []) or [])
        paths = list(getattr(pa, "all_physical_paths", []) or [])
        if not (components or links or paths):
            return

        self.out('physical_architecture "Physical Architecture" {')
        for node in components:
            self.track(node.uuid, node.name)
            self.out(f'  node "{esc(node.name)}" {{')
            self.out(f'    id: "{node.uuid}"')
            self.lines += attr_block("    ", description=getattr(node, "description", None))
            self.out("  }")
            self.count("node")

        # Physical links. ArcLang identifies links by NAME (physical_path
        # involves references link names); the Capella UUID is kept as id:
        # for the reverse name-sync. from:/to: are REQUIRED by the parser —
        # a link is only emitted when both port owners were emitted above.
        for link in links:
            src_owner = getattr(getattr(link, "source", None), "owner", None)
            tgt_owner = getattr(getattr(link, "target", None), "owner", None)
            src_uuid = getattr(src_owner, "uuid", None)
            tgt_uuid = getattr(tgt_owner, "uuid", None)
            if src_uuid not in self.emitted_ids or tgt_uuid not in self.emitted_ids:
                self.count("physical_link_skipped")
                continue
            self.out(f'  link "{esc(link.name)}" {{')
            self.out(f'    id: "{link.uuid}"')
            self.out(f'    from: "{src_uuid}"')
            self.out(f'    to: "{tgt_uuid}"')
            self.out("  }")
            self.emitted_link_names.add(link.name)
            self.count("physical_link")

        # Physical paths: ordered link references BY NAME (compile error if
        # dangling; involves is REQUIRED non-empty, so paths whose links were
        # all skipped are skipped too).
        for path in paths:
            involves = [
                link.name
                for link in (getattr(path, "involved_links", []) or [])
                if link.name in self.emitted_link_names
            ]
            if not involves:
                self.count("physical_path_skipped")
                continue
            self.track(path.uuid, path.name)
            self.out(f'  physical_path "{esc(path.name)}" {{')
            self.out(f'    id: "{path.uuid}"')
            self.lines += ref_list("    ", "involves", involves)
            self.lines += attr_block("    ", description=getattr(path, "description", None))
            self.out("  }")
            self.count("physical_path")
        self.out("}")
        self.out()

    # ---- Data model (Arcadia: Class, Enumeration, DataType, ExchangeItem) --

    @staticmethod
    def is_identifier(name: str) -> bool:
        return bool(re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", name or ""))

    def walk_data_packages(self):
        """Yield every DataPkg of the 4 layers, recursively (deterministic
        document order)."""

        def walk(pkg):
            yield pkg
            for sub in getattr(pkg, "packages", []) or []:
                yield from walk(sub)

        for layer in (self.model.oa, self.model.sa, self.model.la, self.model.pa):
            pkg = getattr(layer, "data_package", None)
            if pkg is not None:
                yield from walk(pkg)

    def emit_data_model(self) -> None:
        classes = list(self.model.search("Class"))
        enumerations = list(self.model.search("Enumeration"))
        datatypes = [
            datatype
            for pkg in self.walk_data_packages()
            for datatype in (getattr(pkg, "datatypes", []) or [])
            if type(datatype).__name__ != "Enumeration"
        ]
        exchange_items = list(self.model.search("ExchangeItem"))
        if not (classes or enumerations or datatypes or exchange_items):
            return

        for class_def in classes:
            self.track(class_def.uuid, class_def.name)
            self.out(f'class "{esc(class_def.name)}" {{')
            self.out(f'  id: "{class_def.uuid}"')
            self.lines += attr_block("  ", description=getattr(class_def, "description", None))
            # Every non-reserved attribute key is a field (name -> type).
            # Attribute keys must be identifiers in the grammar; properties
            # with non-identifier or reserved names are not representable.
            for prop in getattr(class_def, "owned_properties", []) or []:
                prop_type = getattr(getattr(prop, "type", None), "name", None)
                if not self.is_identifier(prop.name) or prop.name in ("id", "description"):
                    continue
                self.out(f'  {prop.name}: "{esc(prop_type or "Unspecified")}"')
                self.count("class_field")
            self.out("}")
            self.count("class")
        if classes:
            self.out()

        for enumeration in enumerations:
            self.track(enumeration.uuid, enumeration.name)
            values = [literal.name for literal in getattr(enumeration, "literals", []) or []]
            self.out(f'enumeration "{esc(enumeration.name)}" {{')
            self.out(f'  id: "{enumeration.uuid}"')
            self.lines += ref_list("  ", "values", values)
            self.out("}")
            self.count("enumeration")
        if enumerations:
            self.out()

        for datatype in datatypes:
            self.track(datatype.uuid, datatype.name)
            self.out(f'data_type "{esc(datatype.name)}" {{')
            self.out(f'  id: "{datatype.uuid}"')
            self.out("}")
            self.count("data_type")
        if datatypes:
            self.out()

        # Exchange items last: their elements: reference emitted classes.
        # Dangling element references are compile ERRORS.
        for item in exchange_items:
            mechanism = getattr(getattr(item, "type", None), "name", None)
            elements = []
            for element in getattr(item, "elements", []) or []:
                target = getattr(element, "abstract_type", None)
                target_uuid = getattr(target, "uuid", None)
                if target_uuid in self.emitted_ids and target_uuid not in elements:
                    elements.append(target_uuid)
            self.track(item.uuid, item.name)
            self.out(f'exchange_item "{esc(item.name)}" {{')
            self.out(f'  id: "{item.uuid}"')
            if mechanism in MECHANISMS:  # UNSET -> omitted (defaults to DATA)
                self.out(f'  mechanism: "{mechanism}"')
            self.lines += ref_list("  ", "elements", elements)
            self.out("}")
            self.count("exchange_item")
        if exchange_items:
            self.out()

    # ---- Modes & States / Scenarios (Arcadia transverse) --------------------

    def emit_state_machines(self) -> None:
        """State machines from every layer (they live on entities, components,
        even ports). ArcLang identifies machines and states by NAME — the AST
        stores no id for them, so this is forward-only (no reverse sync).
        Transition endpoints must be states declared in the same machine."""
        for machine in self.model.search("StateMachine"):
            state_lines: list[str] = []
            emitted_states: set[str] = set()
            transitions = []
            for region in getattr(machine, "regions", []) or []:
                for state in getattr(region, "states", []) or []:
                    if not state.name or state.name in emitted_states:
                        continue  # unnamed pseudostates are not representable
                    keyword = "mode" if type(state).__name__ == "Mode" else "state"
                    state_lines.append(f'  {keyword} "{esc(state.name)}"')
                    emitted_states.add(state.name)
                    self.count(keyword)
                transitions.extend(getattr(region, "transitions", []) or [])

            self.out(f'state_machine "{esc(machine.name)}" {{')
            self.lines += state_lines
            for transition in transitions:
                source = getattr(getattr(transition, "source", None), "name", None)
                target = getattr(getattr(transition, "destination", None), "name", None)
                if source not in emitted_states or target not in emitted_states:
                    self.count("transition_skipped")
                    continue
                # Unresolved triggers are compiler warnings — only emit a
                # trigger that names an element emitted in this file.
                trigger = next(
                    (
                        name
                        for name in (
                            getattr(t, "name", None)
                            for t in (getattr(transition, "triggers", []) or [])
                        )
                        if name and (name in self.emitted_names or name in self.emitted_ids)
                    ),
                    None,
                )
                line = f'  transition "{esc(source)}" -> "{esc(target)}"'
                if trigger:
                    line += f' {{ trigger: "{esc(trigger)}" }}'
                self.out(line)
                self.count("transition")
            self.out("}")
            self.out()
            self.count("state_machine")

    def emit_scenarios(self) -> None:
        """Scenarios (sequence flows) from every layer. Participants are the
        design elements represented by the lifelines (Part -> .type), and are
        compile ERRORS when dangling — only emitted elements participate.
        Messages must connect participants; identified by NAME (no id in the
        AST), so forward-only."""
        for scenario in self.model.search("Scenario"):
            role_element: dict[str, str] = {}
            participants: list[str] = []
            for role in getattr(scenario, "instance_roles", []) or []:
                represented = getattr(getattr(role, "instance", None), "type", None)
                element_uuid = getattr(represented, "uuid", None)
                if element_uuid not in self.emitted_ids:
                    continue
                role_element[role.uuid] = element_uuid
                if element_uuid not in participants:
                    participants.append(element_uuid)
            if not participants:
                self.count("scenario_skipped")
                continue

            self.out(f'scenario "{esc(scenario.name)}" {{')
            self.lines += ref_list("  ", "participants", participants)
            for message in getattr(scenario, "messages", []) or []:

                def covered_element(end) -> str | None:
                    try:
                        return role_element.get(end[0].covered[0].uuid)
                    except (IndexError, TypeError, AttributeError):
                        return None

                source = covered_element(getattr(message, "sending_end", None))
                target = covered_element(getattr(message, "receiving_end", None))
                if not source or not target:
                    self.count("message_skipped")
                    continue
                line = f'  message "{source}" -> "{target}"'
                if message.name:
                    line += f' "{esc(message.name)}"'
                if "ASYNCHRONOUS" in str(getattr(message, "kind", "")):
                    line += ' { type: "async" }'
                self.out(line)
                self.count("message")
            self.out("}")
            self.out()
            self.count("scenario")

    def emit_requirements(self) -> None:
        """Emit Capella requirement extensions as an ArcLang requirements block.

        The requirement UUID becomes the ArcLang requirement name/id;
        `title:` carries the Capella requirement name and `description:`
        carries the requirement text (raw HTML Markup — esc() keeps it
        byte-exact so the reverse path can compare/synchronize it).
        """
        try:
            reqs = list(self.model.search("Requirement"))
        except Exception:
            reqs = []
        if not reqs:
            return

        self.out("requirements {")
        for req in reqs:
            self.emitted_requirements.add(req.uuid)
            self.out(f'  requirement "{req.uuid}" {{')
            text = getattr(req, "text", None)
            self.lines += attr_block(
                "    ",
                title=getattr(req, "name", None),
                description=str(text) if text else None,
            )
            self.out("  }")
            self.count("requirement")
        self.out("}")
        self.out()

    def emit_requirement_traces(self) -> None:
        """Emit requirement<->element relations as ArcLang traces.

        Capella relation types seen on a requirement's `.relations` list
        (`.target` is always the *other* end, seen from the requirement):
          - CapellaOutgoingRelation: requirement -> model element
          - CapellaIncomingRelation: model element -> requirement
          - InternalRelation:        requirement -> requirement (NOT emitted:
            ArcLang traces here link design elements to requirements)
        All are normalized to the ArcLang form
            trace "<element-uuid>" satisfies "<requirement-uuid>"
        (the element satisfies the requirement).

        The strict parser rejects dangling trace endpoints as compile
        ERRORS, so a trace is only emitted when the element UUID was emitted
        in this file AND the requirement UUID was emitted above.
        """
        try:
            reqs = list(self.model.search("Requirement"))
        except Exception:
            return
        seen: set[tuple[str, str]] = set()
        lines: list[str] = []
        for req in reqs:
            if req.uuid not in self.emitted_requirements:
                continue
            for relation in getattr(req, "relations", []) or []:
                target = getattr(relation, "target", None)
                target_uuid = getattr(target, "uuid", None)
                if not target_uuid or target_uuid not in self.emitted_ids:
                    continue  # requirement->requirement or unemitted element
                pair = (target_uuid, req.uuid)
                if pair in seen:
                    continue
                seen.add(pair)
                lines.append(f'trace "{target_uuid}" satisfies "{req.uuid}"')
                self.count("trace")
        if lines:
            # Capella relation order is not stable across model edits —
            # sort for a deterministic, diffable .arc output.
            self.lines += sorted(lines)
            self.out()

    def emit(self) -> str:
        self.emit_model_header()
        self.emit_operational_analysis()
        self.emit_system_analysis()
        self.emit_logical_architecture()
        self.emit_physical_architecture()
        self.emit_data_model()
        self.emit_state_machines()
        self.emit_scenarios()
        self.emit_requirements()
        self.emit_requirement_traces()
        return "\n".join(self.lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("aird", help="Path to the Capella .aird file")
    parser.add_argument("-o", "--output", help="Output .arc path (default: stdout)")
    args = parser.parse_args()

    warnings.filterwarnings("ignore")
    model = capellambse.MelodyModel(args.aird)
    emitter = ArcEmitter(model)
    arc_source = emitter.emit()

    if args.output:
        with open(args.output, "w", encoding="utf-8") as handle:
            handle.write(arc_source)
        summary = ", ".join(f"{v} {k}(s)" for k, v in sorted(emitter.stats.items()))
        print(f"Wrote {args.output}: {summary}")
    else:
        sys.stdout.write(arc_source)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
