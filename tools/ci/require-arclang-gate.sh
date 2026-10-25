#!/usr/bin/env bash
# Make the "ArcLang gate" job a required status check of a GitHub branch.
#
#   tools/ci/require-arclang-gate.sh OWNER/REPO [BRANCH]            print the command, change nothing
#   tools/ci/require-arclang-gate.sh OWNER/REPO [BRANCH] --apply    run it (needs `gh auth login`, admin rights)
#
# It PUTs the branch protection of BRANCH (default main) with the required check
# "ArcLang gate" (the name of the job in .github/workflows/arclang.yml). PUT replaces
# the whole protection: read the current one first (the script prints the command that does)
# and keep what you had. Nothing is run without --apply.
set -euo pipefail

repo="${1:-}"
branch="main"
apply=0
shift || true
for arg in "$@"; do
  case "$arg" in
    --apply) apply=1 ;;
    -*) echo "unknown option $arg" >&2; exit 2 ;;
    *) branch="$arg" ;;
  esac
done
if [[ ! "$repo" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]]; then
  echo "usage: $0 OWNER/REPO [BRANCH] [--apply]" >&2
  exit 2
fi
if [[ ! "$branch" =~ ^[A-Za-z0-9_./-]+$ ]]; then
  echo "not a branch name: $branch" >&2
  exit 2
fi

body='{
  "required_status_checks": { "strict": true, "contexts": ["ArcLang gate"] },
  "enforce_admins": false,
  "required_pull_request_reviews": null,
  "restrictions": null
}'

show() {
  echo "# current protection (keep what you had):"
  echo "gh api repos/$repo/branches/$branch/protection"
  echo "# require the ArcLang gate:"
  echo "gh api --method PUT repos/$repo/branches/$branch/protection --input - <<'JSON'"
  echo "$body"
  echo "JSON"
}

if [[ $apply -eq 0 ]]; then
  show
  echo "# nothing was run; add --apply to run the PUT." >&2
  exit 0
fi
command -v gh > /dev/null || { echo "the GitHub CLI (gh) is needed for --apply" >&2; exit 1; }
echo "$body" | gh api --method PUT "repos/$repo/branches/$branch/protection" --input -
