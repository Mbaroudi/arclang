#!/bin/sh
# Validate a Capella model with Capella itself (headless).
# usage: capella_validate.sh <folder holding the .capella/.aird> <name.aird> <out.json>
set -eu
CAPELLA="${CAPELLA_HOME:-$HOME/tools/capella/Capella.app/Contents/MacOS}/capella"
src="$1"; aird="$2"; out="$3"
work="$(mktemp -d)"; trap 'rm -rf "$work"' EXIT
mkdir -p "$work/projects"; cp -R "$src" "$work/projects/M"
printf '<?xml version="1.0" encoding="UTF-8"?><projectDescription><name>M</name><comment></comment><projects></projects><buildSpec></buildSpec><natures><nature>org.polarsys.capella.project.nature</nature></natures></projectDescription>' > "$work/projects/M/.project"
if [ "${MIGRATE:-0}" = 1 ]; then
  "$CAPELLA" -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid org.polarsys.capella.migration -data "$work/ws-migration" -import "$work/projects/M" -input "/M" > "$work/migration.out" 2>&1 || { tail -20 "$work/migration.out" >&2; exit 4; }
fi
"$CAPELLA" -nosplash -consoleLog -application org.polarsys.capella.core.commandline.core -appid org.polarsys.capella.core.validation.commandline -data "$work/ws" -import "$work/projects/M" -input "/M/$aird" -outputfolder /M/validation -forceoutputfoldercreation -logfile "$work/log.html" > "$work/run.out" 2>&1 || { tail -30 "$work/run.out" >&2; exit 2; }
report="$(find "$work/projects/M/validation" -name validation-results.html | head -1)"
[ -n "$report" ] || { echo "no validation report" >&2; tail -30 "$work/run.out" >&2; exit 3; }
python3 - "$report" "$out" <<'PY'
import sys, re, html, json, collections
t = open(sys.argv[1], errors='replace').read()
rows = re.findall(r'<tr>(.*?)</tr>', t, re.S)
out = collections.Counter(); msgs = collections.defaultdict(list)
for r in rows:
    cells = [html.unescape(re.sub(r'<[^>]+>', '', c)).strip() for c in re.findall(r'<td[^>]*>(.*?)</td>', r, re.S)]
    if len(cells) < 4: continue
    level, rule = cells[1], cells[2]
    out[f'{level} {rule}'] += 1
    if len(msgs[f'{level} {rule}']) < 3: msgs[f'{level} {rule}'].append(cells[0][:200])
json.dump({'counts': out, 'examples': msgs}, open(sys.argv[2], 'w'), indent=1)
lv = collections.Counter()
for k, n in out.items(): lv[k.split()[0]] += n
print(dict(lv)); [print(f'  {n:5d} {k}') for k, n in out.most_common(12)]
PY
