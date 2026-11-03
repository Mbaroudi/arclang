"""Counts of a Capella validation-results.html per level and rule, before vs after."""
import sys, re, html, collections, glob

def parse(path):
    t = open(path, errors='replace').read()
    out = collections.Counter(); msgs = collections.defaultdict(list)
    for r in re.findall(r'<tr>(.*?)</tr>', t, re.S):
        cells = [html.unescape(re.sub(r'<[^>]+>', '', c)).strip() for c in re.findall(r'<td[^>]*>(.*?)</td>', r, re.S)]
        if len(cells) < 4:
            continue
        key = (cells[1], cells[2])
        out[key] += 1
        msgs[key].append(cells[0][:160])
    return out, msgs

root = sys.argv[1]
for m in ['msm_test', 'acc', 'remote_start']:
    rep = {}
    for s in ['before', 'after']:
        f = glob.glob(f'{root}/{s}-{m}/projects/M/validation/**/validation-results.html', recursive=True)
        rep[s] = parse(f[0]) if f else None
    print(f'== {m}')
    if None in rep.values():
        print('  missing report:', [s for s, v in rep.items() if v is None]); continue
    b, a = rep['before'][0], rep['after'][0]
    lv = lambda c: dict(sorted(collections.Counter({k[0]: 0 for k in c} | {}).items())) if False else dict(sum((collections.Counter({k[0]: n}) for k, n in c.items()), collections.Counter()))
    print('  levels before', lv(b), ' after', lv(a))
    for k in sorted(set(b) | set(a)):
        flag = '' if b[k] == a[k] else '   <-- changed'
        print(f'  {k[0]:8s} {k[1]:60s} {b[k]:4d} -> {a[k]:4d}{flag}')
    for k in sorted(set(a)):
        if a[k] > b[k]:
            new = [x for x in rep['after'][1][k] if x not in rep['before'][1][k]]
            for x in new[:6]:
                print('     + ', x)
