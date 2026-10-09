#!/usr/bin/env python3
"""Check the SysML v2 vocabulary ArcLang serves against the OMG pilot.

For each ArcLang model:
  1. `arclang export -f sys-ml` writes the SysML v2 text;
  2. the OMG pilot implementation parses that text and exports its abstract
     syntax as JSON (`%export`), the reference;
  3. `arclang export -f sysml-json` writes the abstract syntax ArcLang reads
     back from the same text, the records served under /api/sysml-v2.
Both are reduced to the same canonical listing of the DECLARED elements:
metaclass, name, short name, owning membership, direction, abstractness,
the qualified names of typing / specialization / redefinition targets
(standard-library elements included), connector ends, the bounds of a stated
multiplicity, and the expression
bound to a feature or asserted by a constraint (operators, literals with
their kind, referents, invoked functions), in ownership order. Any
difference fails the check.

Not compared, because ArcLang does not serve them: the content of library
elements, and what the pilot derives implicitly
(connector end features, the parameter features that carry arguments,
conjugated port definitions, implied specializations).

Setup: the `sysml-pilot` conda environment of tools/sysmlv2_validate.py.
Usage:
    conda activate sysml-pilot
    python tools/sysml_abstract_syntax_check.py [--arclang BIN] [--cache DIR] [--freeze DIR] model.arc [...]
`--cache DIR` keeps the pilot's exports (slow to produce) keyed by the
SHA-256 of the SysML text. `--freeze DIR` writes the SysML text and the
pilot's canonical listing of each model: frozen references that
`cargo test --test sysml_vocabulary` checks without the pilot.
"""
import argparse
import base64
import difflib
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile

# Elements the pilot creates without a declaration in the text.
IMPLICIT_TYPES = {"Multiplicity", "ConjugatedPortDefinition", "FlowEnd", "Feature"}
IMPLICIT_MEMBERSHIPS = {"ParameterMembership", "ReturnParameterMembership", "EndFeatureMembership", "SubjectMembership"}
SPECIALIZATIONS = {
    "FeatureTyping": ":",
    "Subclassification": ":>",
    "Subsetting": ":>",
    "Redefinition": ":>>",
    "ReferenceSubsetting": "references",
}


def canonical(elements):
    """Canonical listing of the declared elements of an abstract syntax.

    Accepts the pilot's `%export` ({identity, payload} items) and ArcLang's
    records alike: both use the property names of the KerML metamodel.
    """
    by = {(x["identity"]["@id"] if "payload" in x else x["@id"]): x.get("payload", x) for x in elements}

    def kind(ref):
        return by.get(ref["@id"], {}).get("@type", "?")

    def qualified(ref):
        element = by.get(ref["@id"], {})
        chain = element.get("chainingFeature") or []
        if chain:  # a feature chain is known by the path it spells
            return ".".join(by.get(c["@id"], {}).get("declaredName") or "?" for c in chain)
        return element.get("qualifiedName") or "<%s>" % element.get("@type", "?")

    def end(element, role):
        related = element.get(role) or []
        if related:
            return ",".join(qualified(r) for r in related)
        return element.get("arclang:%sPath" % role, "")

    def expression(ref):
        """An expression as text: kind of each node, operator, referents."""
        node = by[ref["@id"]]
        metaclass = node["@type"]
        arguments = [expression(a) for a in node.get("argument") or []]
        if metaclass.startswith("Literal"):
            return "%s(%s)" % (metaclass[len("Literal"):], json.dumps(node.get("value"), ensure_ascii=False))
        if metaclass == "FeatureReferenceExpression":
            referent = node.get("referent")
            return "ref(%s)" % (qualified(referent) if referent else "?" + node.get("arclang:unresolvedTarget", ""))
        if metaclass == "FeatureChainExpression":
            target = node.get("targetFeature")
            written = node.get("arclang:targetPath", "")
            # A multi-step path is an anonymous chain feature for the pilot.
            path = written if "." in written else (qualified(target) if target else "?" + written)
            return "chain(%s, %s)" % (arguments[0], path)
        if metaclass == "InvocationExpression":
            function = node.get("instantiatedType")
            return "call(%s)" % ", ".join([qualified(function) if function else "?"] + arguments)
        if metaclass == "OperatorExpression":
            return "op(%s)" % ", ".join([node.get("operator") or "?"] + arguments)
        return "<%s>" % metaclass

    def declared(element):
        metaclass = element["@type"]
        if metaclass in IMPLICIT_TYPES or metaclass.endswith("Expression") or metaclass.startswith("Literal"):
            return False
        if metaclass == "ReferenceUsage" and not element.get("declaredName"):
            return False
        membership = element.get("owningMembership")
        return not (membership and kind(membership) in IMPLICIT_MEMBERSHIPS)

    def line(element, is_root):
        membership = element.get("owningMembership")
        parts = [
            element["@type"],
            "name=%s" % (element.get("declaredName") or "-"),
            "short=%s" % (element.get("declaredShortName") or "-"),
            "via=%s" % ("-" if is_root or not membership else kind(membership)),
        ]
        if element.get("direction"):
            parts.append("dir=%s" % element["direction"])
        if element.get("isAbstract"):
            parts.append("abstract")
        relationships = []
        for ref in element.get("ownedRelationship", []):
            relationship = by[ref["@id"]]
            symbol = SPECIALIZATIONS.get(relationship["@type"])
            if symbol and not relationship.get("isImplied"):
                target = ",".join(qualified(t) for t in relationship.get("target", []))
                relationships.append("%s %s" % (symbol, target or "?" + relationship.get("arclang:unresolvedTarget", "")))
        parts += sorted(relationships)
        for ref in element.get("ownedRelationship", []):
            relationship = by[ref["@id"]]
            if relationship["@type"] == "FeatureValue":
                parts.append("%s %s" % ("default=" if relationship.get("isDefault") else "=", expression(relationship["value"])))
            elif relationship["@type"] == "ResultExpressionMembership":
                parts.append("result %s" % expression(relationship["ownedResultExpression"]))
        if element["@type"] == "SatisfyRequirementUsage":
            parts.append("req=%s by=%s" % (qualified(element["satisfiedRequirement"]), qualified(element["satisfyingFeature"])))
        elif element["@type"] == "Dependency":
            parts.append("client=%s supplier=%s" % (end(element, "client") or end(element, "source"), end(element, "supplier") or end(element, "target")))
        elif "connectorEnd" in element or "arclang:sourcePath" in element:
            source, target = end(element, "source"), end(element, "target")
            if source or target:
                parts.append("src=%s tgt=%s" % (source, target))
        if element["@type"] == "MultiplicityRange":
            # `[4]` has one bound, `[1..*]` two; the last is the upper bound.
            parts.append("bounds=%s" % ",".join(expression(b) for b in element.get("bound") or []))
        if element["@type"] == "Documentation":
            parts.append("body=%s" % json.dumps(" ".join((element.get("body") or "").split()), ensure_ascii=False))
        return " ".join(parts)

    lines = []

    def show(identity, depth):
        element = by[identity]
        if not declared(element):
            return
        lines.append("  " * depth + line(element, depth == 0))
        members = [m["@id"] for m in element.get("ownedMember", [])]
        # The pilot lists dependencies among the owned relationships.
        members += [
            r["@id"] for r in element.get("ownedRelationship", [])
            if by[r["@id"]]["@type"] == "Dependency" and r["@id"] not in members
        ]
        dependencies = [m for m in members if by[m]["@type"] == "Dependency"]
        for member in members:
            if member not in dependencies:
                show(member, depth + 1)
        # Where a dependency stands among its siblings carries no meaning.
        start = len(lines)
        for member in dependencies:
            show(member, depth + 1)
        lines[start:] = sorted(lines[start:])

    roots = [i for i, e in by.items() if e["@type"] == "Package" and "::" not in (e.get("qualifiedName") or "::")]
    for root in roots:
        show(root, 0)
    return lines


class Pilot:
    """A session with the OMG pilot implementation (its Jupyter kernel)."""

    def __init__(self):
        prefix = os.environ.get("CONDA_PREFIX")
        if prefix:  # the pilot needs the environment's JDK (>= 21)
            os.environ["PATH"] = os.path.join(prefix, "lib", "jvm", "bin") + os.pathsep + os.environ.get("PATH", "")
        from jupyter_client import KernelManager

        self.manager = KernelManager(kernel_name="sysml")
        self.manager.start_kernel(stderr=subprocess.DEVNULL)
        self.client = self.manager.client()
        self.client.start_channels()
        self.client.wait_for_ready(timeout=120)
        # Parser and validator errors of the last `run`.
        self.errors = []

    def run(self, code):
        message_id = self.client.execute(code)
        results = []
        self.errors = []
        while True:
            message = self.client.get_iopub_msg(timeout=600)
            if message.get("parent_header", {}).get("msg_id") != message_id:
                continue
            content = message["content"]
            if message["msg_type"] == "status" and content.get("execution_state") == "idle":
                break
            if message["msg_type"] in ("execute_result", "display_data"):
                results.append(content["data"])
            elif message["msg_type"] == "stream" and content.get("name") == "stderr":
                self.errors += [l.strip() for l in content["text"].splitlines() if l.strip().startswith("ERROR:")]
            elif message["msg_type"] == "error":
                raise RuntimeError(content.get("evalue", "pilot error"))
        self.client.get_shell_msg(timeout=60)
        return results

    def abstract_syntax(self, sysml):
        package = re.search(r"^package (\S+)", sysml, re.M).group(1)
        self.run(sysml)
        if self.errors:
            raise RuntimeError("the pilot rejects the text: " + "; ".join(self.errors[:5]))
        for data in self.run("%export " + package):
            found = re.search(r'base64,([^"]+)"', data.get("text/html", ""))
            if found:
                return json.loads(base64.b64decode(found.group(1)))
        raise RuntimeError("the pilot exported nothing for package %s" % package)

    def close(self):
        self.client.stop_channels()
        self.manager.shutdown_kernel(now=True)


def export(arclang, model, fmt):
    with tempfile.TemporaryDirectory() as scratch:
        out = os.path.join(scratch, "out")
        done = subprocess.run([arclang, "export", model, "-f", fmt, "-o", out], capture_output=True, text=True)
        if done.returncode != 0:
            raise RuntimeError("arclang export -f %s %s: %s" % (fmt, model, done.stderr.strip()))
        with open(out, encoding="utf-8") as handle:
            return handle.read()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("models", nargs="+")
    parser.add_argument("--arclang", default="target/debug/arclang")
    parser.add_argument("--cache")
    parser.add_argument(
        "--freeze",
        metavar="DIR",
        help="write each model's SysML text and the pilot's canonical listing to DIR "
        "(the frozen references of tests/sysml_vocabulary.rs)",
    )
    args = parser.parse_args()

    pilot = None
    failed = 0
    try:
        for model in args.models:
            sysml = export(args.arclang, model, "sys-ml")
            ours = canonical(json.loads(export(args.arclang, model, "sysml-json")))
            cached = args.cache and os.path.join(args.cache, hashlib.sha256(sysml.encode()).hexdigest() + ".json")
            if cached and os.path.exists(cached):
                with open(cached, encoding="utf-8") as handle:
                    reference = json.load(handle)
            else:
                pilot = pilot or Pilot()
                reference = pilot.abstract_syntax(sysml)
                if cached:
                    os.makedirs(args.cache, exist_ok=True)
                    with open(cached, "w", encoding="utf-8") as handle:
                        json.dump(reference, handle)
            theirs = canonical(reference)
            if args.freeze:
                stem = os.path.join(args.freeze, os.path.splitext(os.path.basename(model))[0])
                os.makedirs(args.freeze, exist_ok=True)
                with open(stem + ".sysml", "w", encoding="utf-8") as handle:
                    handle.write(sysml)
                with open(stem + ".pilot.txt", "w", encoding="utf-8") as handle:
                    handle.write("\n".join(theirs) + "\n")
            if ours == theirs:
                print("OK   %s (%d declared elements)" % (model, len(ours)))
            else:
                failed += 1
                print("FAIL %s" % model)
                for diff in difflib.unified_diff(theirs, ours, "pilot", "arclang", lineterm="", n=0):
                    print("    " + diff[:240])
    finally:
        if pilot:
            pilot.close()
    print("%d model(s) checked, %d differ from the pilot" % (len(args.models), failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
