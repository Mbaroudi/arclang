#!/usr/bin/env python3
"""Build spec/sysml_library_index.json with the OMG pilot implementation.

The SysML v2 export refers to standard-library elements by name: `String`,
`DurationValue`, `[kg]`, `:>> unitConversion`, `DataFunctions::max`. What
each name designates (qualified name, metaclass) is the pilot's to say, not
ours to guess: this tool gives the pilot a package that uses every library
name the exporter can write (`arclang metamodel --format
sysml-library-probe`) and records what each reference resolved to.

The index is what lets /api/sysml-v2 serve references to library elements.
Regenerate it when the exporter learns a new unit, type or function:
    conda activate sysml-pilot
    python tools/sysml_library_index.py [--arclang BIN] [--output spec/sysml_library_index.json]
"""
import argparse
import json
import os
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sysml_abstract_syntax_check import Pilot  # noqa: E402

PROBE = "ArcLangLibraryProbe"


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--arclang", default="target/debug/arclang")
    parser.add_argument("--output", default="spec/sysml_library_index.json")
    args = parser.parse_args()

    probe = subprocess.run(
        [args.arclang, "metamodel", "--format", "sysml-library-probe"], capture_output=True, text=True, check=True
    ).stdout
    pilot = Pilot()
    try:
        elements = pilot.abstract_syntax(probe)
    finally:
        pilot.close()
    by = {x["identity"]["@id"]: x["payload"] for x in elements}

    found = {}

    def record(reference, role):
        element = by.get(reference["@id"])
        if element is None:
            raise SystemExit("the pilot's export does not hold a referenced element (%s)" % role)
        qualified = element.get("qualifiedName")
        if not qualified:
            raise SystemExit("a %s reference designates an unnamed %s" % (role, element["@type"]))
        if qualified == PROBE or qualified.startswith(PROBE + "::"):
            return
        entry = found.setdefault(qualified, {
            "qualifiedName": qualified,
            "metaclass": element["@type"],
            "name": element.get("name"),
            "shortName": element.get("shortName"),
            "roles": [],
        })
        if role not in entry["roles"]:
            entry["roles"].append(role)

    for element in by.values():
        metaclass = element["@type"]
        if element.get("isImplied"):
            continue
        if metaclass == "FeatureTyping":
            record(element["type"], "type")
        elif metaclass == "Redefinition":
            record(element["redefinedFeature"], "redefinition")
        elif metaclass == "FeatureReferenceExpression" and element.get("referent"):
            record(element["referent"], "value")
        elif metaclass == "InvocationExpression" and element.get("instantiatedType"):
            record(element["instantiatedType"], "function")

    index = {
        "generatedBy": "tools/sysml_library_index.py",
        "reference": "OMG SysML v2 pilot implementation (jupyter-sysml-kernel)",
        "elements": [dict(entry, roles=sorted(entry["roles"])) for _, entry in sorted(found.items())],
    }
    with open(args.output, "w", encoding="utf-8") as handle:
        json.dump(index, handle, indent=2, ensure_ascii=False)
        handle.write("\n")
    print("%d library elements written to %s" % (len(index["elements"]), args.output))


if __name__ == "__main__":
    main()
