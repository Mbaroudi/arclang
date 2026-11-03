#!/usr/bin/env python3
"""Every (parent kind, feature, child kind) and (kind, attribute) our export
writes, checked against what real Capella 7 models write: EMF ignores an
unknown feature in silence, Capella then reports unrelated errors.

usage: shape_check.py <export.capella> <corpus> [reference.json]
The reference is built from the Capella 7 models of the corpus on the first
run and kept in the given file (default: shapes_ref.json beside the corpus).
Exits 1 when the export writes a shape no Capella 7 model has."""
import sys, glob, json, os, collections, xml.etree.ElementTree as ET
S = os.path.dirname(os.path.abspath(__file__))
XSI = '{http://www.w3.org/2001/XMLSchema-instance}type'
K = lambda e: (e.get(XSI) or e.tag.split('}')[-1]).split('.')[-1]
corpus = sys.argv[2]
ref = sys.argv[3] if len(sys.argv) > 3 else os.path.join(os.path.dirname(os.path.abspath(corpus)), 'shapes_ref.json')
def shapes(path):
    root = ET.parse(path).getroot(); feats = set(); attrs = set()
    for e in root.iter():
        for a in e.attrib:
            if a not in (XSI,) and not a.startswith('{'): attrs.add((K(e), a))
        for c in e:
            feats.add((K(e), c.tag.split('}')[-1], K(c)))
    return feats, attrs
if not os.path.exists(ref):
    F, A = set(), set()
    for f in glob.glob(f'{corpus}/**/*.capella', recursive=True):
        try:
            if 'modeller/7.0.0' not in open(f, errors='replace').read(3000): continue
            fe, at = shapes(f); F |= fe; A |= at
        except Exception: pass
    json.dump({'f': sorted(map(list, F)), 'a': sorted(map(list, A))}, open(ref, 'w'))
R = json.load(open(ref)); RF = set(map(tuple, R['f'])); RA = set(map(tuple, R['a']))
fe, at = shapes(sys.argv[1])
# Any Capella element may hold property values, whether or not a model of the corpus does.
bad_f = sorted(x for x in fe if x not in RF and not (x[1] == 'ownedPropertyValues' and x[2].endswith('PropertyValue'))); bad_a = sorted(x for x in at if x not in RA)
for x in bad_f:
    alt = sorted({t for (p, t, c) in RF if p == x[0] and c == x[2]})
    print('feature', x, 'capella uses:', alt[:4])
for x in bad_a: print('attribute', x)
print(len(bad_f), 'unknown features,', len(bad_a), 'unknown attributes')
sys.exit(1 if bad_f or bad_a else 0)
