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


class ArcEmitter:
    def __init__(self, model: capellambse.MelodyModel):
        self.model = model
        self.lines: list[str] = []
        self.stats: dict[str, int] = {}
        self.emitted_components: set[str] = set()

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
            self.out(f'  actor "{esc(actor.name)}" {{')
            self.out(f'    id: "{actor.uuid}"')
            self.lines += attr_block("    ", description=getattr(actor, "description", None))
            self.out("  }")
            self.count("actor")
        for entity in entities:
            self.out(f'  entity "{esc(entity.name)}" {{')
            self.out(f'    id: "{entity.uuid}"')
            for activity in getattr(entity, "activities", []) or []:
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
            self.out(f'  operational_activity "{esc(activity.name)}" {{')
            self.out(f'    id: "{activity.uuid}"')
            self.out("  }")
            self.count("activity")
        self.out("}")
        self.out()

    def emit_system_analysis(self) -> None:
        sa = self.model.sa
        functions = list(getattr(sa, "all_functions", []) or [])
        if not functions:
            return

        self.out('system_analysis "System Analysis" {')
        for function in functions:
            self.out(f'  function "{esc(function.name)}" {{')
            self.out(f'    id: "{function.uuid}"')
            self.lines += attr_block("    ", description=getattr(function, "description", None))
            self.out("  }")
            self.count("function")
        self.out("}")
        self.out()

    def emit_component(self, component, indent: str) -> None:
        self.emitted_components.add(component.uuid)
        self.out(f'{indent}component "{esc(component.name)}" {{')
        self.out(f'{indent}  id: "{component.uuid}"')
        self.lines += attr_block(
            indent + "  ",
            description=getattr(component, "description", None),
        )
        for function in getattr(component, "allocated_functions", []) or []:
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
        self.out("}")
        self.out()

    def emit_physical_architecture(self) -> None:
        pa = self.model.pa
        try:
            roots = list(pa.component_package.components)
        except Exception:
            roots = list(getattr(pa, "all_components", []) or [])
        if not roots:
            return

        self.out('physical_architecture "Physical Architecture" {')
        for node in roots:
            self.out(f'  node "{esc(node.name)}" {{')
            self.out(f'    id: "{node.uuid}"')
            self.lines += attr_block("    ", description=getattr(node, "description", None))
            self.out("  }")
            self.count("node")
        self.out("}")
        self.out()

    def emit(self) -> str:
        self.emit_model_header()
        self.emit_operational_analysis()
        self.emit_system_analysis()
        self.emit_logical_architecture()
        self.emit_physical_architecture()
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
