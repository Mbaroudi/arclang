#!/bin/sh
# The smart lock on an Arduino Uno: export the C of the model, build the
# sketch of uno_hil/ around it and flash the board. Needs arduino-cli with
# the arduino:avr core (`arduino-cli core install arduino:avr`), and an
# `arclang` on the PATH (or ARCLANG=path/to/arclang).
#
#   examples/cesam/smart-lock/uno/flash.sh [serial port]
#
# Then: arclang hil examples/cesam/smart-lock/uno/smart_lock_bench.arc --port <serial port>
set -e
here=$(cd "$(dirname "$0")" && pwd)
model="$here/smart_lock_bench.arc"
arclang=${ARCLANG:-arclang}
port=${1:-$(arduino-cli board list 2>/dev/null | awk '/arduino:avr:uno/ { print $1; exit }')}
if [ -z "$port" ]; then
  echo "no Arduino Uno found: give its serial port (arduino-cli board list)" >&2
  exit 1
fi
out=$(mktemp -d)
"$arclang" export "$model" -f c -o "$out"
cp "$out/smart_door_lock.c" "$out/smart_door_lock.h" "$out/host/harness.c" "$here/uno_hil/"
rm -rf "$out"
# The harness has a main: the Arduino core has its own, the harness's is called from loop().
arduino-cli compile --fqbn arduino:avr:uno --build-property "compiler.c.extra_flags=-Dmain=arclang_harness_main" "$here/uno_hil"
arduino-cli upload -p "$port" --fqbn arduino:avr:uno "$here/uno_hil"
echo
echo "flashed. Now: $arclang hil $model --port $port"
