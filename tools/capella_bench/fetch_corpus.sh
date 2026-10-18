#!/bin/sh
# Fetch the Capella corpus of the bench, each repository at the revision
# corpus.txt pins (one shallow fetch per repository).
# usage: fetch_corpus.sh <destination>
set -eu
dest="${1:?usage: fetch_corpus.sh <destination>}"
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$dest"
while read -r dir url rev; do
  [ -z "$dir" ] && continue
  target="$dest/$dir"
  if [ -d "$target/.git" ] && [ "$(git -C "$target" rev-parse HEAD)" = "$rev" ]; then
    continue
  fi
  rm -rf "$target"; mkdir -p "$target"
  git -C "$target" init -q
  git -C "$target" remote add origin "$url"
  git -C "$target" fetch -q --depth 1 origin "$rev"
  git -C "$target" -c advice.detachedHead=false checkout -q FETCH_HEAD
  echo "fetched $dir at ${rev%"${rev#????????????}"}"
done < "$here/corpus.txt"
