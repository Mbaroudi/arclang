#!/usr/bin/env bash
# Pushes the registry tokens kept in a local .env file to the GitHub
# repository secrets read by .github/workflows/release.yml:
#   VSCE_PAT  <- VSCE_PAT   (Visual Studio Marketplace, Azure DevOps token)
#   OVSX_PAT  <- OVSX_PAT, or VSX_KEY when OVSX_PAT is absent (Open VSX token)
# Values are piped straight into `gh secret set`; nothing is printed.
#
# Usage: editors/vscode/scripts/sync-registry-secrets.sh [path/to/.env] [owner/repo]
set -euo pipefail

env_file="${1:-$(git rev-parse --show-toplevel)/.env}"
repo="${2:-$(gh repo view --json nameWithOwner --jq .nameWithOwner)}"

[ -r "$env_file" ] || { echo "no readable env file at $env_file" >&2; exit 1; }

read_var() {  # read_var NAME -> value on stdout, empty when absent
  grep -E "^$1=" "$env_file" | head -1 | cut -d= -f2- | tr -d '"'"'"' \r'
}

vsce="$(read_var VSCE_PAT)"
ovsx="$(read_var OVSX_PAT)"
if [ -z "$ovsx" ]; then
  ovsx="$(read_var VSX_KEY)"
  if [ -n "$ovsx" ]; then
    printf 'OVSX_PAT=%s\n' "$ovsx" >> "$env_file"
    echo "OVSX_PAT added to $env_file (same value as VSX_KEY)"
  fi
fi

status=0
for pair in "VSCE_PAT:$vsce" "OVSX_PAT:$ovsx"; do
  name="${pair%%:*}"; value="${pair#*:}"
  if [ -z "$value" ]; then
    echo "$name: not found in $env_file, skipped" >&2; status=1; continue
  fi
  printf '%s' "$value" | gh secret set "$name" --repo "$repo" --app actions
  echo "$name: set on $repo (${#value} characters)"
done

gh secret list --repo "$repo"
exit $status
