#!/bin/sh
# Export every active example to SysML v2 and validate each export with the
# OMG SysML v2 pilot implementation. Exit 1 if the pilot rejects any file.
#
# Requires the `sysml-pilot` conda environment (see tools/sysmlv2_validate.py)
# to be ACTIVE, and a built `arclang` binary (cargo build).
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$root/target/debug/arclang"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

[ -x "$bin" ] || { echo "build first: cargo build" >&2; exit 2; }

find "$root/examples" -name '*.arc' -not -path '*/legacy/*' | sort | while read -r model; do
    case "$model" in
        */multifile/*) grep -q '^import ' "$model" || continue ;;  # fragments are exported through their root
    esac
    name="$(echo "${model#$root/examples/}" | tr '/' '_')"
    "$bin" export "$model" -f sys-ml -o "$out/${name%.arc}.sysml" >/dev/null
done

python "$root/tools/sysmlv2_validate.py" "$out"/*.sysml
