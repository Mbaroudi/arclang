#!/usr/bin/env python3
"""ArcLang -> Capella bridge (M3 proof of concept).

Applies an ArcLang model (the AST JSON produced by `arclang export -f json`)
onto a Capella model, matching elements by their UUIDs (which capella2arc
preserves). Element names are synchronized; elements present in the .arc but
unknown to the Capella model are reported (creation is not attempted yet).

Together with capella2arc.py this enables the round-trip test:
Capella -> .arc -> Capella must be a zero diff when nothing was edited.

Usage:
    python arc2capella.py model.json target.aird [--dry-run]
"""

from __future__ import annotations

import argparse
import json
import sys
import warnings

import capellambse


def iter_arc_elements(doc: dict):
    """Yield (uuid_or_id, name, kind) for every identified element in the AST JSON."""

    def walk_component(comp, kind):
        yield comp.get("id"), comp.get("name"), kind
        for sub in comp.get("sub_components", []) or []:
            yield from walk_component(sub, kind)
        for func in comp.get("functions", []) or []:
            attrs = func.get("attributes", {}) or {}
            fid = _attr_string(attrs.get("id"))
            if fid:
                yield fid, func.get("name"), "function"

    for oa in doc.get("operational_analysis", []) or []:
        for actor in oa.get("actors", []) or []:
            attrs = actor.get("attributes", {}) or {}
            aid = actor.get("id") or _attr_string(attrs.get("id"))
            yield aid, actor.get("name"), "actor"
        for entity in oa.get("entities", []) or []:
            yield entity.get("id"), entity.get("name"), "entity"
            for activity in entity.get("activities", []) or []:
                yield activity.get("id"), activity.get("name"), "activity"
        for activity in oa.get("activities", []) or []:
            yield activity.get("id"), activity.get("name"), "activity"

    for sa in doc.get("system_analysis", []) or []:
        for function in sa.get("functions", []) or []:
            attrs = function.get("attributes", {}) or {}
            fid = _attr_string(attrs.get("id")) or function.get("id")
            yield fid, function.get("name"), "function"

    for la in doc.get("logical_architecture", []) or []:
        for comp in la.get("components", []) or []:
            yield from walk_component(comp, "component")

    for pa in doc.get("physical_architecture", []) or []:
        for node in pa.get("nodes", []) or []:
            yield node.get("id"), node.get("name"), "node"


def _attr_string(value):
    """AttributeValue JSON is {"String": "..."} / {"Number": ...} / ..."""
    if isinstance(value, dict):
        return value.get("String")
    if isinstance(value, str):
        return value
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("arc_json", help="AST JSON from `arclang export -f json`")
    parser.add_argument("aird", help="Target Capella .aird")
    parser.add_argument("--dry-run", action="store_true", help="Report only, don't save")
    args = parser.parse_args()

    warnings.filterwarnings("ignore")
    with open(args.arc_json, encoding="utf-8") as handle:
        doc = json.load(handle)

    model = capellambse.MelodyModel(args.aird)

    matched = renamed = unknown = skipped = 0
    for uuid, name, kind in iter_arc_elements(doc):
        if not uuid or not name:
            continue
        # Only UUID-shaped ids can be matched back into Capella
        if len(uuid) != 36 or uuid.count("-") != 4:
            skipped += 1
            continue
        try:
            element = model.by_uuid(uuid)
        except KeyError:
            print(f"  unknown in Capella: {kind} '{name}' ({uuid})")
            unknown += 1
            continue
        matched += 1
        if element.name != name:
            print(f"  rename: {kind} '{element.name}' -> '{name}' ({uuid})")
            element.name = name
            renamed += 1

    print(
        f"matched: {matched}, renamed: {renamed}, unknown: {unknown}, "
        f"non-uuid ids skipped: {skipped}"
    )

    if args.dry_run:
        print("dry-run: not saving")
    else:
        model.save()
        print(f"saved {args.aird}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
