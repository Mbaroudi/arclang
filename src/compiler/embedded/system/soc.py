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
#            "tcp_port", "seed",
#            "blocks": [{"name", "verilog", "base", "size", "address_bits", "clock_hz"}]}
#
# clock_hz is the bus's (the processor's); a block on another clock is in a
# clock domain of its own, f<hz>, reached through a Wishbone clock domain
# crossing.

import json
import os
import sys

from migen import ClockDomain, ClockSignal, Instance, ResetSignal, Signal

from litex.build.generic_platform import Pins, Subsignal
from litex.build.io import CRG
from migen.genlib.resetsync import AsyncResetSynchronizer
from litex.gen import LiteXModule
from litex.soc.cores.clock import ECP5PLL
from litex.soc.integration.builder import Builder
from litex.soc.integration.common import get_mem_data
from litex.soc.integration.soc import SoCRegion
from litex.soc.integration.soc_core import SoCCore
from litex.soc.interconnect import wishbone

MAIN_RAM = 0x40000000


def domains(config):
    """The blocks' clocks other than the bus's: {domain name: Hz}."""
    return {"f%d" % b["clock_hz"]: b["clock_hz"] for b in config["blocks"] if b["clock_hz"] != config["clock_hz"]}


def domain_of(config, block):
    return "sys" if block["clock_hz"] == config["clock_hz"] else "f%d" % block["clock_hz"]


class SimCRG(LiteXModule):
    """As litex_sim: sys from its clock, LiteX's power-on reset; each other
    domain from a clock of its own, reset with sys."""

    def __init__(self, platform, config):
        self.crg = CRG(platform.request("sys_clk"))
        for name in domains(config):
            cd = ClockDomain(name)
            setattr(self, "cd_" + name, cd)
            self.comb += cd.clk.eq(platform.request("clk_" + name))
            self.specials += AsyncResetSynchronizer(cd, ResetSignal("sys"))


class BoardCRG(LiteXModule):
    """The board's oscillator; a PLL when a clock differs from it."""

    def __init__(self, platform, config):
        clock_hz = config["clock_hz"]
        self.cd_sys = ClockDomain()
        others = domains(config)
        osc = platform.request(platform.default_clk_name)
        osc_hz = round(1e9 / platform.default_clk_period)
        if clock_hz == osc_hz and not others:
            self.comb += self.cd_sys.clk.eq(osc)
            # Held in reset for the first cycles after configuration.
            count = Signal(4, reset=15)
            self.sync += count.eq(count - (count != 0))
            self.comb += self.cd_sys.rst.eq(count != 0)
        else:
            self.pll = pll = ECP5PLL()
            pll.register_clkin(osc, osc_hz)
            pll.create_clkout(self.cd_sys, clock_hz)
            for name, hz in others.items():
                cd = ClockDomain(name)
                setattr(self, "cd_" + name, cd)
                pll.create_clkout(cd, hz)


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
            self.add_block(platform, block, domain_of(config, block))

    def add_block(self, platform, block, domain):
        bus = wishbone.Interface(data_width=32, address_width=32, addressing="word")
        port = bus
        if domain != "sys":
            # The block's side of a clock domain crossing.
            port = wishbone.Interface(data_width=32, address_width=32, addressing="word")
            self.submodules += wishbone.ClockDomainCrossing(bus, port, cd_from="sys", cd_to=domain)
        bits = block["address_bits"]
        self.specials += Instance(
            block["name"] + "_wb",
            i_clk=ClockSignal(domain),
            i_rst=ResetSignal(domain),
            i_wb_adr=port.adr[:bits],
            i_wb_dat_w=port.dat_w,
            o_wb_dat_r=port.dat_r,
            i_wb_sel=port.sel,
            i_wb_cyc=port.cyc,
            i_wb_stb=port.stb,
            i_wb_we=port.we,
            o_wb_ack=port.ack,
            o_wb_err=port.err,
            o_irq=Signal(name=block["name"] + "_irq"),
        )
        platform.add_source(block["verilog"])
        self.bus.add_slave(block["name"], bus, SoCRegion(origin=block["base"], size=block["size"], cached=False))


def sim_platform(config):
    from litex.build.sim import SimPlatform

    io = [("clk_" + name, 0, Pins(1)) for name in domains(config)] + [
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
        platform = sim_platform(config)
        crg = SimCRG(platform, config)
    elif what == "board":
        platform = board_platform(config)
        crg = BoardCRG(platform, config)
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
        for name, hz in domains(config).items():
            sim.add_clocker("clk_" + name, freq_hz=hz)
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
