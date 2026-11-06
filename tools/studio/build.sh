#!/bin/sh
# Build ArcLang Studio from these sources, the chain's images first when
# they are not built yet (an hour or two the first time; then the studio
# alone, minutes). docs/studio.md.
#   tools/studio/build.sh [tag]        (default: arclang-studio:dev)
set -e
here=$(cd "$(dirname "$0")/../.." && pwd)
tag=${1:-arclang-studio:dev}
built() { docker image inspect "$1" >/dev/null 2>&1; }
built arclang-embedded:1 || docker build -t arclang-embedded:1 "$here/tools/embedded"
built arclang-embedded-soc:1 || docker build -t arclang-embedded-soc:1 "$here/tools/embedded/soc"
built arclang-embedded-fpga:1 || docker build -t arclang-embedded-fpga:1 "$here/tools/embedded/fpga"
built arclang-embedded-system:1 || docker build -t arclang-embedded-system:1 "$here/tools/embedded/system"
docker build -f "$here/tools/studio/Dockerfile" -t "$tag" "$here"
echo "built $tag: docker run --rm -p 127.0.0.1:3000:3000 -v \"\$PWD\":/home/arclang/workspace $tag"
