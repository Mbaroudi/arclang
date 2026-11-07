"""Split the single region of chosen states of a .capella into several regions.

usage: split_regions.py <in.capella> <out.capella> STATE_ID=grp1,grp2,... ...
where each group is a '+'-joined list of sub-state ids (the first group also
keeps the region's pseudo states). Transitions move to the region of their source.
"""
import sys, uuid
from lxml import etree

XSI = '{http://www.w3.org/2001/XMLSchema-instance}type'
NS = uuid.UUID('6f1e3c1a-9c1e-4d0e-9a0c-3b3e0e2d5a77')

def main(src, dst, specs):
    tree = etree.parse(src)
    by_id = {e.get('id'): e for e in tree.iter() if e.get('id')}
    for spec in specs:
        sid, groups = spec.split('=')
        groups = [g.split('+') for g in groups.split(',')]
        state = by_id[sid]
        region = [c for c in state if c.tag == 'ownedRegions'][0]
        states = [c for c in region if c.tag == 'ownedStates']
        transitions = [c for c in region if c.tag == 'ownedTransitions']
        placed = {s for g in groups for s in g}
        pseudo = [s.get('id') for s in states if s.get('id') not in placed]
        groups[0] = pseudo + groups[0]
        home = {s: k for k, g in enumerate(groups) for s in g}
        regions = [region]
        for k in range(1, len(groups)):
            r = etree.Element('ownedRegions')
            r.set(XSI, region.get(XSI))
            r.set('id', str(uuid.uuid5(NS, f'{sid}:region:{k}')))
            r.set('name', f'Region {k + 1}')
            region.addnext(r) if k == 1 else regions[-1].addnext(r)
            regions.append(r)
        region.set('name', 'Region 1')
        for s in states:
            regions[home[s.get('id')]].append(s)
        for t in transitions:
            regions[home[t.get('source').lstrip('#')]].append(t)
        for k, r in enumerate(regions):
            r.set('involvedStates', ' '.join('#' + s for s in groups[k]))
        state.set('referencedStates', ' '.join('#' + s for g in groups for s in g))
    tree.write(dst, xml_declaration=True, encoding='UTF-8')

main(sys.argv[1], sys.argv[2], sys.argv[3:])
