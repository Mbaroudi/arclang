#!/bin/sh
# Build this Zephyr application for the LiteX VexRiscv SoC and run it in
# Renode on ArcLang's platform of that SoC (renode/), the console on stdout.
#   ./run.sh                 build (west) and run until Ctrl-C
#   ./run.sh build           build only: build/zephyr/zephyr.elf
# Needs west and the Zephyr SDK (ZEPHYR_BASE set) and renode on the PATH:
# ArcLang's image tools/embedded/soc has them.
# On a board: load build/zephyr/zephyr.bin with the LiteX BIOS
# (litex_term --kernel build/zephyr/zephyr.bin /dev/ttyUSB1), or put it in
# the SPI flash; the SoC must have the peripherals of
# boards/litex_vexriscv.overlay at those addresses.
set -e
here=$(cd "$(dirname "$0")" && pwd)
west build -p auto -b litex_vexriscv -d "$here/build" "$here"
[ "${1:-}" = build ] && exit 0
cat > "$here/build/run.resc" <<RESC
include @$here/renode/ArcLangSJA1000.cs
include @$here/renode/RegisterADC.cs
include @$here/renode/LiteXPWM.cs
include @$here/renode/LiteXWatchdog.cs
mach create
machine LoadPlatformDescription @$here/renode/arclang_litex_vexriscv.repl
sysbus LoadELF @$here/build/zephyr/zephyr.elf
showAnalyzer sysbus.uart Antmicro.Renode.Analyzers.LoggingUartAnalyzer
start
RESC
exec renode --disable-gui --console --plain -e "include @$here/build/run.resc"
