#!/usr/bin/env python3
"""The Capella round trip over the whole corpus: every model of it is
imported, exported back to Capella and imported again, and the two imports
are compared declaration by declaration (roundtrip.py). The bench fails
when a model does not come back identical, when a model the baseline does
not list as refused fails to import, or when fewer models or declarations
are measured than the baseline holds.

usage: bench.py <corpus> <work> [--json results.json] [--jobs N]
The binary is $ARCLANG, else target/release/arclang of the repository."""
import argparse, concurrent.futures, hashlib, json, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
SUMMARY = re.compile(r'declarations: (\d+) in the first import; same (\d+), changed (\d+), lost (\d+), new (\d+)')
TRACES = re.compile(r'traces: (\d+) in the first import; same (\d+), lost (\d+), new (\d+)')


def models(corpus):
    """The distinct .capella files of the corpus, by content, relative to it."""
    seen, found = set(), []
    for root, _, files in os.walk(corpus):
        if '/.git' in root or '/expected-output' in root:
            continue
        for name in sorted(files):
            if name.endswith('.capella'):
                path = os.path.join(root, name)
                digest = hashlib.sha1(open(path, 'rb').read()).hexdigest()[:12]
                if digest not in seen:
                    seen.add(digest)
                    found.append((os.path.relpath(path, corpus), digest))
    return sorted(found)


def measure(corpus, work, model):
    rel, digest = model
    p = subprocess.run([sys.executable, f'{HERE}/roundtrip.py', os.path.join(corpus, rel), os.path.join(work, digest)],
                       capture_output=True, text=True, timeout=900)
    out = p.stdout + p.stderr
    row = {'file': rel, 'hash': digest}
    m, t = SUMMARY.search(out), TRACES.search(out)
    if m:
        row.update(dict(zip(['total', 'same', 'changed', 'lost', 'new'], map(int, m.groups()))))
        if t:
            row.update(dict(zip(['traces', 'traces_same', 'traces_lost', 'traces_new'], map(int, t.groups()))))
        row['examples'] = re.findall(r'^  e\.g\. (.*)$', out, re.M)[:5]
    else:
        row['error'] = out.strip()[-600:]
    return row


def identical(row):
    return 'error' not in row and row['same'] == row['total'] and row['new'] == 0 \
        and row.get('traces_lost', 0) == 0 and row.get('traces_new', 0) == 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('corpus'); parser.add_argument('work')
    parser.add_argument('--json'); parser.add_argument('--jobs', type=int, default=os.cpu_count() or 2)
    args = parser.parse_args()
    baseline = json.load(open(f'{HERE}/baseline.json'))
    all_models = models(args.corpus)
    with concurrent.futures.ThreadPoolExecutor(args.jobs) as pool:
        rows = list(pool.map(lambda m: measure(args.corpus, args.work, m), all_models))
    if args.json:
        json.dump(rows, open(args.json, 'w'), indent=1)

    failures = []
    measured = [r for r in rows if r['file'] not in baseline['refused']]
    for row in rows:
        refused = row['file'] in baseline['refused']
        if refused and 'error' not in row:
            failures.append(f"{row['file']}: listed as refused but imports now; update baseline.json")
        elif not refused and not identical(row):
            detail = row.get('error') or '; '.join(row.get('examples', [])) or f"same {row['same']} of {row['total']}"
            failures.append(f"{row['file']}: {detail[:400]}")
    same = sum(r.get('same', 0) for r in measured)
    total = sum(r.get('total', 0) for r in measured)
    exact = sum(1 for r in measured if identical(r))
    print(f'models: {len(rows)} ({len(rows) - len(measured)} refused as expected), identical {exact} of {len(measured)}')
    print(f'declarations: identical {same} of {total}')
    if len(measured) < baseline['models']:
        failures.append(f"{len(measured)} models measured, the baseline has {baseline['models']}: is the corpus complete?")
    if total < baseline['declarations']:
        failures.append(f"{total} declarations measured, the baseline has {baseline['declarations']}")
    for failure in failures:
        print('FAIL', failure)
    sys.exit(1 if failures else 0)


if __name__ == '__main__':
    main()
