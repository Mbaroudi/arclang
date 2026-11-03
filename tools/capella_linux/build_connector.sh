#!/bin/sh
# Build the Capella-Simulink connector's bundles from its sources with javac,
# against the Capella 7.1 Linux install, into a dropin.
# usage: build.sh <connector checkout> <capella dir> <work dir>
set -eu
SRC="$1"; CAP="$2"; WORK="$3"
Q=v$(date +%Y%m%d%H%M%S)
OUT="$CAP/dropins/csc/plugins"
rm -rf "$WORK/bin" "$OUT"; mkdir -p "$WORK/bin" "$OUT"
CP=$(ls "$CAP"/plugins/*.jar | grep -v '[.]source_' | tr '\n' ':')
for d in "$CAP"/plugins/*/; do CP="$CP:$d"; done
BUNDLES="sources/plugins/org.eclipse.capella.simulink.core.model
sources/plugins/org.eclipse.capella.simulink.transformation
capella/plugins/org.eclipse.capella.simulink.capella.m2m"
for b in $BUNDLES; do
  name=$(basename "$b"); dir="$SRC/$b"; bin="$WORK/bin/$name"
  mkdir -p "$bin"
  srcs=""; for s in src utils; do [ -d "$dir/$s" ] && srcs="$srcs $dir/$s"; done
  find $srcs -name '*.java' > "$WORK/$name.files"
  javac --release 17 -nowarn -encoding UTF-8 -proc:none -cp "$CP" -d "$bin" @"$WORK/$name.files" 2>&1 | grep -v '^Note:' || true
  # non-Java files of the source folders, then bin.includes
  for s in $srcs; do (cd "$s" && find . -type f ! -name '*.java' -exec sh -c 'mkdir -p "$1/$(dirname "$2")" && cp "$2" "$1/$2"' _ "$bin" {} \;); done
  for inc in model resources schema plugin.xml plugin.properties about.html; do [ -e "$dir/$inc" ] && cp -R "$dir/$inc" "$bin/"; done
  sed "s/\.qualifier/.$Q/" "$dir/META-INF/MANIFEST.MF" > "$WORK/$name.MF"
  v=$(grep '^Bundle-Version:' "$WORK/$name.MF" | awk '{print $2}' | tr -d '\r')
  jar cfm "$OUT/${name}_$v.jar" "$WORK/$name.MF" -C "$bin" .
  CP="$CP:$bin"
  echo "built $name $v ($(find "$bin" -name '*.class' | wc -l | tr -d ' ') classes)"
done
