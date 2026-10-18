#!/usr/bin/env python3
"""Capella -> ArcLang -> Capella -> ArcLang for one model: what the second
import loses or changes against the first, per declaration keyword, and
with --validate Capella's own validation of the export.

usage: roundtrip.py <model.capella> <work dir> [--validate]
The binary is $ARCLANG, else target/release/arclang of the repository."""
import subprocess, sys, os, re, json, collections, difflib, shutil
S = os.path.dirname(os.path.abspath(__file__))
B = os.environ.get('ARCLANG', os.path.join(S, '..', '..', 'target', 'release', 'arclang'))
src, work = sys.argv[1], sys.argv[2]
validate = '--validate' in sys.argv
os.makedirs(work, exist_ok=True)
stem = os.path.basename(src)[:-len('.capella')]
a = f'{work}/a.arc'; out = f'{work}/export/{stem}.capella'; b = f'{work}/b.arc'
def run(*args):
    r = subprocess.run(args, capture_output=True, text=True); return r.returncode, r.stdout + r.stderr
code, msg = run(B, 'import', '-f', 'capella', src, '-o', a)
if code: print('first import failed:', msg[-300:]); sys.exit(1)
shutil.rmtree(f'{work}/export', ignore_errors=True)
code, msg = run(B, 'export', a, '-f', 'capella', '-o', out)
if code: print('export failed:', msg[-500:]); sys.exit(1)
notes = [l for l in msg.splitlines() if 'note:' in l]
code, msg = run(B, 'import', '-f', 'capella', out, '-o', b)
if code: print('re-import failed:', msg[-800:]); sys.exit(1)
def decls(path):
    """Declarations by (keyword, id), with their lines."""
    lines = open(path).read().splitlines(); result = {}; stack = []
    for line in lines:
        m = re.match(r'(\s*)([a-z_]+)(?: (?:in|out|inout))? (?:"[^"]*"\s*)*\{(.*)', line)
        if m:
            idm = re.search(r'id: "([^"]+)"', line)
            stack.append([m.group(2), idm.group(1) if idm else None, [line.strip()]])
            if line.rstrip().endswith('}'):
                k, i, body = stack.pop(); result[(k, i or body[0])] = body
            continue
        if stack:
            idm = re.match(r'\s*id: "([^"]+)"', line)
            if idm and stack[-1][1] is None: stack[-1][1] = idm.group(1)
            stack[-1][2].append(line.strip())
            if line.strip() == '}':
                k, i, body = stack.pop(); result[(k, i or body[0])] = body
    return result
def normal(body):
    # A view's `shows` is a set: its order is not meaning.
    out = []
    for line in body:
        if line.startswith('shows: ['):
            items = sorted(re.findall(r'"[^"]*"', line)); line = 'shows: [' + ', '.join(items) + ']'
        out.append(line)
    return out
A, Bd = decls(a), decls(b)
A = {k: normal(v) for k, v in A.items()}; Bd = {k: normal(v) for k, v in Bd.items()}
by = collections.defaultdict(lambda: collections.Counter())
examples = collections.defaultdict(list)
for key, body in A.items():
    k = key[0]
    if key not in Bd: by[k]['lost'] += 1; examples[(k,'lost')].append(body[0][:120]); continue
    # The order of attributes inside a declaration is not meaning.
    if body[0] == Bd[key][0] and sorted(body[1:]) == sorted(Bd[key][1:]): by[k]['same'] += 1
    else:
        by[k]['changed'] += 1
        d = [l for l in difflib.unified_diff(body, Bd[key], lineterm='', n=0) if l[:1] in '+-' and not l.startswith(('+++','---'))]
        examples[(k,'changed')].append(' | '.join(d)[:240])
for key in Bd:
    if key not in A: by[key[0]]['new'] += 1; examples[(key[0],'new')].append(Bd[key][0][:120])
tot = collections.Counter()
for k, c in by.items(): tot.update(c)
print(f"declarations: {sum(tot.values())-tot['new']} in the first import; same {tot['same']}, changed {tot['changed']}, lost {tot['lost']}, new {tot['new']}")
for k, c in sorted(by.items(), key=lambda kv: -sum(kv[1].values())):
    print(f"  {k:28s} " + ' '.join(f'{x}={c[x]}' for x in ('same','changed','lost','new') if c[x]))
# Traces are lines, not blocks: compared as sets.
def traces(path):
    return set(l.strip() for l in open(path) if l.startswith('trace '))
ta, tb = traces(a), traces(b)
print(f"traces: {len(ta)} in the first import; same {len(ta & tb)}, lost {len(ta - tb)}, new {len(tb - ta)}")
for t in sorted(ta - tb)[:2]: print('  e.g. trace lost:', t[:160])
for t in sorted(tb - ta)[:2]: print('  e.g. trace new:', t[:160])
for (k, what), ex in sorted(examples.items()):
    print(f"  e.g. {k} {what}: {ex[0]}")
if notes: print('export notes:', len(notes)); [print(' ', n.strip()[:160]) for n in notes[:8]]
if validate:
    code, msg = run('sh', f'{S}/capella_validate.sh', os.path.dirname(out), stem + '.aird', f'{work}/validation.json')
    print('capella validation:', msg.strip()[:600] if msg.strip() else code)
