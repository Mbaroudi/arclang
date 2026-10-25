#!/usr/bin/env bash
# The workbench page in a browser, against the real compiler: every palette
# tool of every view, the editing gestures, the changes of what is drawn,
# every toolbar tool.
# Needs Playwright (PLAYWRIGHT=/path/to/node_modules/playwright if not resolvable)
# and a release build of the compiler (cargo build --release at the repo root).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
ext="$here/../.."
root="$(mktemp -d)"
trap 'kill "${server:-0}" 2>/dev/null || true; rm -rf "$root"' EXIT
(cd "$ext" && npm run -s build)
cp -R "$ext/media/viewer" "$root/viewer"
cp "$ext/dist/webview.js" "$ext/media/workbench.css" "$ext/media/panes.css" "$ext/media/ai.css" "$ext/media/arrange.css" "$here/e2e.html" "$root/" # zorder hook: arrange.css
"$here/../../../../target/release/arclang" metamodel --format json > "$root/metamodel.json"
"$here/../../../../target/release/arclang" workbench "$ext/media/reference.arc" > "$root/snapshot.json"
cp "$here/static.html" "$root/index.html"
PORT=${PORT:-8766} node "$here/server.js" "$root" "$ext/media/reference.arc" & server=$!
sleep 1
PORT=${PORT:-8766} node "$here/palette_test.js"
PORT=${PORT:-8766} node "$here/gestures_test.js"
PORT=${PORT:-8766} node "$here/editing_test.js"
PORT=${PORT:-8766} node "$here/panes_test.js"
node "$here/toolbar_test.js" "$root"
# tables hook: the table view, on tables the compiler computed.
cp "$ext/dist/tables.js" "$ext/media/tables.css" "$here/tables.html" "$root/"
fixture="$here/../../../../tests/fixtures/tables/braking.arc"
for t in "Requirement coverage:coverage" "Satisfaction:satisfaction" "Interfaces:interfaces" "ICD:icd"; do
  "$here/../../../../target/release/arclang" table "$fixture" --name "${t%%:*}" > "$root/${t##*:}.json"
done
node "$here/tables_test.js" "$root"
