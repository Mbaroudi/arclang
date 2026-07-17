#!/usr/bin/env python3
"""ArcLang -> Capella bridge.

Applies an ArcLang model (the AST JSON produced by `arclang export -f json`)
onto a Capella model, matching elements by their UUIDs (which capella2arc
preserves). Synchronized on the reverse path:

  - element names (actors, entities, activities, functions, components, nodes)
  - element descriptions (where the forward path emits them)
  - requirements: title -> Capella name, description -> Capella text
  - --create-missing: logical components present in the .arc but unknown to
    Capella (UUID not found, or non-UUID id) are created under their matched
    .arc parent, or under the LA root component package
  - --delete-missing: logical components present in the Capella component
    package subtree but absent from the .arc are deleted (explicit flag only)

Together with capella2arc.py this enables the round-trip test:
Capella -> .arc -> Capella must be a zero diff when nothing was edited.

Usage:
    python arc2capella.py model.json target.aird [--dry-run]
                          [--create-missing] [--delete-missing]
"""

from __future__ import annotations

import argparse
import json
import sys
import warnings

import capellambse

UUID_LEN = 36
UUID_DASHES = 4


def is_uuid(value: str | None) -> bool:
    return bool(value) and len(value) == UUID_LEN and value.count("-") == UUID_DASHES


def _attr_string(value):
    """AttributeValue JSON is {"String": "..."} / {"Number": ...} / ..."""
    if isinstance(value, dict):
        return value.get("String")
    if isinstance(value, str):
        return value
    return None


def iter_arc_elements(doc: dict):
    """Yield (uuid_or_id, name, kind, description) for every identified element.

    Covers, in addition to the structural elements:
      - OA:  operational processes, communication means
      - SA:  missions, capabilities, functional chains
      - LA:  capability realizations, functional chains
      - PA:  physical links (id: attribute), physical paths
      - top: classes, data types/enumerations, exchange items
    State machines and scenarios carry NO id in the ArcLang AST (identified
    by name only) — they cannot be synchronized back and are skipped.
    Sync is name+description only (no create/delete for these kinds).
    """

    def id_named(items, kind):
        """Elements serialized as {id, name, attributes{description?}}."""
        for item in items or []:
            attrs = item.get("attributes", {}) or {}
            yield (
                item.get("id"),
                item.get("name"),
                kind,
                _attr_string(attrs.get("description")),
            )

    def walk_component(comp, kind):
        attrs = comp.get("attributes", {}) or {}
        yield comp.get("id"), comp.get("name"), kind, _attr_string(attrs.get("description"))
        for sub in comp.get("sub_components", []) or []:
            yield from walk_component(sub, kind)
        for func in comp.get("functions", []) or []:
            fattrs = func.get("attributes", {}) or {}
            fid = _attr_string(fattrs.get("id"))
            if fid:
                yield fid, func.get("name"), "function", None

    for oa in doc.get("operational_analysis", []) or []:
        for actor in oa.get("actors", []) or []:
            attrs = actor.get("attributes", {}) or {}
            aid = actor.get("id") or _attr_string(attrs.get("id"))
            yield aid, actor.get("name"), "actor", _attr_string(attrs.get("description"))
        for entity in oa.get("entities", []) or []:
            yield entity.get("id"), entity.get("name"), "entity", None
            for activity in entity.get("activities", []) or []:
                yield activity.get("id"), activity.get("name"), "activity", None
        for activity in oa.get("activities", []) or []:
            yield activity.get("id"), activity.get("name"), "activity", None
        yield from id_named(oa.get("processes"), "operational_process")
        # Communication means: OperationalExchange JSON — the name is the
        # `label`, the Capella UUID is the `id` attribute.
        for mean in oa.get("communication_means", []) or []:
            attrs = mean.get("attributes", {}) or {}
            yield (
                _attr_string(attrs.get("id")),
                mean.get("label"),
                "communication_means",
                _attr_string(attrs.get("description")),
            )

    for sa in doc.get("system_analysis", []) or []:
        for function in sa.get("functions", []) or []:
            attrs = function.get("attributes", {}) or {}
            fid = _attr_string(attrs.get("id")) or function.get("id")
            yield fid, function.get("name"), "function", _attr_string(attrs.get("description"))
        yield from id_named(sa.get("missions"), "mission")
        yield from id_named(sa.get("capabilities"), "capability")
        yield from id_named(sa.get("functional_chains"), "functional_chain")

    for la in doc.get("logical_architecture", []) or []:
        for comp in la.get("components", []) or []:
            yield from walk_component(comp, "component")
        yield from id_named(la.get("capability_realizations"), "capability_realization")
        yield from id_named(la.get("functional_chains"), "functional_chain")

    for pa in doc.get("physical_architecture", []) or []:
        for node in pa.get("nodes", []) or []:
            attrs = node.get("attributes", {}) or {}
            yield node.get("id"), node.get("name"), "node", _attr_string(attrs.get("description"))
        # Physical links: identified by name in ArcLang; UUID in `id` attr.
        for link in pa.get("links", []) or []:
            attrs = link.get("attributes", {}) or {}
            yield (
                _attr_string(attrs.get("id")),
                link.get("name"),
                "physical_link",
                _attr_string(attrs.get("description")),
            )
        yield from id_named(pa.get("paths"), "physical_path")

    # Top-level data model. DataType JSON carries no attributes map, so only
    # the name can be synchronized for data types/enumerations.
    yield from id_named(doc.get("classes"), "class")
    for data_type in doc.get("data_types", []) or []:
        yield data_type.get("id"), data_type.get("name"), "data_type", None
    for item in doc.get("exchange_items", []) or []:
        yield item.get("id"), item.get("name"), "exchange_item", None


def iter_arc_requirements(doc: dict):
    """Yield (uuid, title, text) for every requirement in the AST JSON."""
    for sa in doc.get("system_analysis", []) or []:
        for req in sa.get("requirements", []) or []:
            attrs = req.get("attributes", {}) or {}
            yield (
                req.get("id"),
                _attr_string(attrs.get("title")),
                _attr_string(attrs.get("description")),
            )


def iter_arc_component_tree(doc: dict):
    """Yield (comp_dict, parent_id_or_None) in tree order (parents first)."""

    def walk(comp, parent_id):
        yield comp, parent_id
        for sub in comp.get("sub_components", []) or []:
            yield from walk(sub, comp.get("id"))

    for la in doc.get("logical_architecture", []) or []:
        for comp in la.get("components", []) or []:
            yield from walk(comp, None)


class Sync:
    def __init__(self, model: capellambse.MelodyModel):
        self.model = model
        self.created_ids: set[str] = set()
        self.stats = {
            "matched": 0,
            "renamed": 0,
            "described": 0,
            "req_synced": 0,
            "created": 0,
            "deleted": 0,
            "unknown": 0,
            "skipped": 0,
        }

    # ---- attribute sync ---------------------------------------------------

    def sync_elements(self, doc: dict) -> None:
        for uuid, name, kind, description in iter_arc_elements(doc):
            if not uuid or not name:
                continue
            if uuid in self.created_ids:
                continue  # freshly created by --create-missing, already in sync
            if not is_uuid(uuid):
                self.stats["skipped"] += 1
                continue
            try:
                element = self.model.by_uuid(uuid)
            except KeyError:
                print(f"  unknown in Capella: {kind} '{name}' ({uuid})")
                self.stats["unknown"] += 1
                continue
            self.stats["matched"] += 1
            if element.name != name:
                print(f"  rename: {kind} '{element.name}' -> '{name}' ({uuid})")
                element.name = name
                self.stats["renamed"] += 1
            if description is not None:
                current = str(element.description) if element.description else ""
                if current != description:
                    print(f"  description: {kind} '{name}' updated ({uuid})")
                    element.description = description
                    self.stats["described"] += 1

    def sync_requirements(self, doc: dict) -> None:
        for uuid, title, text in iter_arc_requirements(doc):
            if not is_uuid(uuid):
                self.stats["skipped"] += 1
                continue
            try:
                req = self.model.by_uuid(uuid)
            except KeyError:
                print(f"  unknown in Capella: requirement ({uuid})")
                self.stats["unknown"] += 1
                continue
            self.stats["matched"] += 1
            changed = False
            if title is not None and req.name != title:
                print(f"  requirement title: '{req.name}' -> '{title}' ({uuid})")
                req.name = title
                changed = True
            if text is not None:
                current = str(req.text) if req.text else ""
                if current != text:
                    print(f"  requirement text updated ({uuid})")
                    req.text = text
                    changed = True
            if changed:
                self.stats["req_synced"] += 1

    # ---- structural sync --------------------------------------------------

    def _find(self, uuid: str | None):
        if not is_uuid(uuid):
            return None
        try:
            return self.model.by_uuid(uuid)
        except KeyError:
            return None

    def create_missing_components(self, doc: dict) -> None:
        """Create logical components from the .arc that Capella doesn't know.

        Verified capellambse 0.8.1 incantation:
            parent.components.create(name=...)   # LogicalComponent + its Part
        works both on a LogicalComponent parent and on the LA root
        LogicalComponentPkg (model.la.component_package.components).
        """
        created: dict[str, object] = {}  # arc-id -> created Capella object
        for comp, parent_id in iter_arc_component_tree(doc):
            arc_id = comp.get("id")
            name = comp.get("name")
            if not arc_id or not name or self._find(arc_id) is not None:
                continue
            parent = created.get(parent_id) or self._find(parent_id)
            if parent is None:
                parent = self.model.la.component_package
            new = parent.components.create(name=name)
            attrs = comp.get("attributes", {}) or {}
            description = _attr_string(attrs.get("description"))
            if description:
                new.description = description
            created[arc_id] = new
            self.created_ids.add(arc_id)
            parent_label = getattr(parent, "name", "LA component package")
            print(f"  created: '{name}' under '{parent_label}': {arc_id} -> {new.uuid}")
            self.stats["created"] += 1

    def delete_missing_components(self, doc: dict) -> None:
        """Delete Capella logical components (component_package subtree)
        that are absent from the .arc.

        Verified capellambse 0.8.1 incantation (both steps required — a bare
        `components.remove()` leaves a dangling Part behind):
            for part in component.representing_parts:
                part.parent.owned_parts.remove(part)
            component.parent.components.remove(component)
        Deleting a parent also deletes its XML subtree, so once a component
        is deleted we do not recurse into its children.
        """
        arc_uuids = {
            comp.get("id")
            for comp, _ in iter_arc_component_tree(doc)
            if is_uuid(comp.get("id"))
        }

        def prune(container_owner):
            for component in list(container_owner.components):
                if component.uuid not in arc_uuids:
                    print(f"  delete: '{component.name}' ({component.uuid})")
                    for part in list(component.representing_parts):
                        part.parent.owned_parts.remove(part)
                    container_owner.components.remove(component)
                    self.stats["deleted"] += 1
                else:
                    prune(component)

        prune(self.model.la.component_package)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("arc_json", help="AST JSON from `arclang export -f json`")
    parser.add_argument("aird", help="Target Capella .aird")
    parser.add_argument("--dry-run", action="store_true", help="Report only, don't save")
    parser.add_argument(
        "--create-missing",
        action="store_true",
        help="Create logical components present in the .arc but unknown to Capella",
    )
    parser.add_argument(
        "--delete-missing",
        action="store_true",
        help="DELETE logical components present in Capella but absent from the .arc",
    )
    args = parser.parse_args()

    warnings.filterwarnings("ignore")
    with open(args.arc_json, encoding="utf-8") as handle:
        doc = json.load(handle)

    model = capellambse.MelodyModel(args.aird)
    sync = Sync(model)

    if args.create_missing:
        sync.create_missing_components(doc)
    sync.sync_elements(doc)
    sync.sync_requirements(doc)
    if args.delete_missing:
        sync.delete_missing_components(doc)

    stats = sync.stats
    print(
        f"matched: {stats['matched']}, renamed: {stats['renamed']}, "
        f"descriptions updated: {stats['described']}, "
        f"requirements updated: {stats['req_synced']}, "
        f"created: {stats['created']}, deleted: {stats['deleted']}, "
        f"unknown: {stats['unknown']}, non-uuid ids skipped: {stats['skipped']}"
    )

    if args.dry_run:
        print("dry-run: not saving")
    else:
        model.save()
        print(f"saved {args.aird}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
