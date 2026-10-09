#!/bin/sh
# Check, for every active example, that the SysML v2 vocabulary ArcLang
# serves agrees with the abstract syntax the OMG pilot implementation derives
# from the same export. Exit 1 on any difference.
#
# Requires the `sysml-pilot` conda environment (see tools/sysmlv2_validate.py)
# to be ACTIVE, and a built `arclang` binary (cargo build).
# SYSML_PILOT_CACHE=<dir> reuses the pilot's exports between runs.
set -e
root="$(cd "$(dirname "$0")/.." && pwd)"
bin="$root/target/debug/arclang"
[ -x "$bin" ] || { echo "build first: cargo build" >&2; exit 2; }

set --
for model in $(find "$root/examples" -name '*.arc' -not -path '*/legacy/*' | sort); do
    case "$model" in
        */multifile/*) grep -q '^import ' "$model" || continue ;;  # fragments are checked through their root
    esac
    set -- "$@" "$model"
done

if [ -n "$SYSML_PILOT_CACHE" ]; then
    python "$root/tools/sysml_abstract_syntax_check.py" --arclang "$bin" --cache "$SYSML_PILOT_CACHE" "$@"
else
    python "$root/tools/sysml_abstract_syntax_check.py" --arclang "$bin" "$@"
fi
