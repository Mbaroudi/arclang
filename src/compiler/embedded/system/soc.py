#!/usr/bin/env python3
# The SoC of an ArcLang model's hardware/software split, written by
# `arclang soc`: a VexRiscv, its RAM holding the Zephyr application from
# reset (no BIOS, no ROM), a UART for the console, the timer Zephyr needs,
# and every generated block on the Wishbone bus at the address its driver
# uses. The same SoC is simulated cycle by cycle (Verilator, LiteX's
# simulator) and built into a bitstream for a board.
#
#   python soc.py soc.json csr      # the SoC's map (csr.json), for Zephyr's overlay
#   python soc.py soc.json sim      # the Verilator simulator, console on TCP
#   python soc.py soc.json board    # the bitstream (Yosys, nextpnr-ecp5, ecppack)
#
# soc.json: {"name", "board", "device", "clock_hz", "ram_bytes", "image",
#            "tcp_port", "seed", "blocks": [{"name", "verilog", "base", "size", "address_bits"}]}

import json
import os
import sys

from migen import ClockDomain, ClockSignal, Instance, ResetSignal, Signal

from litex.build.generic_platform import Pins, Subsignal
from litex.build.io import CRG
from litex.gen import LiteXModule
from litex.soc.cores.clock import ECP5PLL
from litex.soc.integration.builder import Builder
from litex.soc.integration.common import get_mem_data
from litex.soc.integration.soc import SoCRegion
from litex.soc.integration.soc_core import SoCCore
from litex.soc.interconnect import wishbone

MAIN_RAM = 0x40000000


class BoardCRG(LiteXModule):
    """The board's oscillator; a PLL when the node's clock differs."""

    def __init__(self, platform, clock_hz):
        self.cd_sys = ClockDomain()
        osc = platform.request(platform.default_clk_name)
        osc_hz = round(1e9 / platform.default_clk_period)
        if clock_hz == osc_hz:
            self.comb += self.cd_sys.clk.eq(osc)
            # Held in reset for the first cycles after configuration.
            count = Signal(4, reset=15)
            self.sync += count.eq(count - (count != 0))
            self.comb += self.cd_sys.rst.eq(count != 0)
        else:
            self.pll = pll = ECP5PLL()
            pll.register_clkin(osc, osc_hz)
            pll.create_clkout(self.cd_sys, clock_hz)


class ArcLangSoC(SoCCore):
    def __init__(self, platform, config, crg, with_image):
        image = []
        if with_image:
            image = get_mem_data(config["image"], endianness="little")
        SoCCore.__init__(
            self,
            platform,
            clk_freq=config["clock_hz"],
            ident="ArcLang " + config["name"],
            cpu_type="vexriscv",
            cpu_variant="standard",
            cpu_reset_address=MAIN_RAM,
            integrated_rom_size=0,
            integrated_sram_size=0,
            integrated_main_ram_size=config["ram_bytes"],
            integrated_main_ram_init=image,
            uart_name="sim" if platform.name == "sim" else "serial",
            with_timer=True,
            timer_uptime=True,
        )
        self.crg = crg
        for block in config["blocks"]:
            self.add_block(platform, block)

    def add_block(self, platform, block):
        bus = wishbone.Interface(data_width=32, address_width=32, addressing="word")
        bits = block["address_bits"]
        self.specials += Instance(
            block["name"] + "_wb",
            i_clk=ClockSignal("sys"),
            i_rst=ResetSignal("sys"),
            i_wb_adr=bus.adr[:bits],
            i_wb_dat_w=bus.dat_w,
            o_wb_dat_r=bus.dat_r,
            i_wb_sel=bus.sel,
            i_wb_cyc=bus.cyc,
            i_wb_stb=bus.stb,
            i_wb_we=bus.we,
            o_wb_ack=bus.ack,
            o_wb_err=bus.err,
            o_irq=Signal(name=block["name"] + "_irq"),
        )
        platform.add_source(block["verilog"])
        self.bus.add_slave(block["name"], bus, SoCRegion(origin=block["base"], size=block["size"], cached=False))


def sim_platform():
    from litex.build.sim import SimPlatform

    io = [
        ("sys_clk", 0, Pins(1)),
        ("serial", 0,
            Subsignal("source_valid", Pins(1)),
            Subsignal("source_ready", Pins(1)),
            Subsignal("source_data", Pins(8)),
            Subsignal("sink_valid", Pins(1)),
            Subsignal("sink_ready", Pins(1)),
            Subsignal("sink_data", Pins(8)),
        ),
    ]
    return SimPlatform("sim", io)


def board_platform(config):
    """The board's LiteX platform, for the node's part."""
    board, device = config["board"], config["device"].upper()
    if board == "ulx3s":
        from litex_boards.platforms import radiona_ulx3s

        # The part without its package and speed: LFE5U-25F, LFE5U-85F...
        return radiona_ulx3s.Platform(device="-".join(device.split("-")[:2]), revision="2.0", toolchain="trellis")
    if board in ("colorlight_i5", "colorlight_i9"):
        from litex_boards.platforms import colorlight_i5

        return colorlight_i5.Platform(board=board[-2:], revision="7.0" if board.endswith("i5") else "7.2", toolchain="trellis")
    if board == "ecp5_evn":
        from litex_boards.platforms import lattice_ecp5_evn

        return lattice_ecp5_evn.Platform(toolchain="trellis")
    sys.exit("arclang soc: no board '%s'" % board)


def main():
    config_path, what = sys.argv[1], sys.argv[2]
    with open(config_path) as f:
        config = json.load(f)
    out = os.path.dirname(os.path.abspath(config_path))
    if what in ("csr", "sim"):
        platform = sim_platform()
        # As litex_sim: the clock alone; the reset is LiteX's power-on reset.
        crg = CRG(platform.request("sys_clk"))
    elif what == "board":
        platform = board_platform(config)
        crg = BoardCRG(platform, config["clock_hz"])
    else:
        sys.exit("arclang soc: csr, sim or board, not '%s'" % what)
    soc = ArcLangSoC(platform, config, crg, with_image=what != "csr")
    builder = Builder(
        soc,
        output_dir=os.path.join(out, what),
        compile_software=False,
        compile_gateware=what != "csr",
        csr_json=os.path.join(out, "csr.json") if what == "csr" else None,
    )
    if what == "csr":
        builder.build(run=False)
    elif what == "sim":
        from litex.build.sim.config import SimConfig

        sim = SimConfig()
        sim.add_clocker("sys_clk", freq_hz=config["clock_hz"])
        sim.add_module("serial2tcp", "serial", args={"port": config["tcp_port"]})
        builder.build(sim_config=sim, run=False, opt_level="O2", threads=1, interactive=False)
        # Compiled here, run by the bench (gateware/obj_dir/Vsim, from gateware/).
        import subprocess

        gateware = os.path.join(out, what, "gateware")
        done = subprocess.run(["bash", "build_sim.sh"], cwd=gateware, capture_output=True, text=True)
        if done.returncode != 0:
            sys.exit("arclang soc: Verilator failed:\n" + "\n".join((done.stdout + done.stderr).splitlines()[-30:]))
    else:
        builder.build(seed=config.get("seed", 1))


if __name__ == "__main__":
    main()
