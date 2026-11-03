#!/bin/sh
# Build the command lines of bundle/ (arclang.diag) into the Capella install's dropins.
# usage: build_bundle.sh <capella dir> <work dir of build_connector.sh>
set -eu
HERE=$(cd "$(dirname "$0")" && pwd); CAP="$1"; CSC="$2"
V=1.0.0.v$(date +%Y%m%d%H%M%S)
CP=$(ls "$CAP"/plugins/*.jar | grep -v '[.]source_' | tr '\n' ':')
for b in "$CSC"/bin/*/; do CP="$CP:$b"; done
OUT=$(mktemp -d); mkdir -p "$OUT/bin"
javac --release 17 -nowarn -cp "$CP" -d "$OUT/bin" "$HERE"/bundle/src/arclang/diag/*.java
cp "$HERE/bundle/plugin.xml" "$OUT/bin/"
sed "s/^Bundle-Version:.*/Bundle-Version: $V/" "$HERE/bundle/META-INF/MANIFEST.MF" > "$OUT/MANIFEST.MF"
mkdir -p "$CAP/dropins/arclang/plugins"; rm -f "$CAP"/dropins/arclang/plugins/arclang.diag_*.jar
jar cfm "$CAP/dropins/arclang/plugins/arclang.diag_$V.jar" "$OUT/MANIFEST.MF" -C "$OUT/bin" .
rm -rf "$OUT"; echo "arclang.diag_$V.jar"
