# The system stage of ArcLang's embedded chain: the processor and the
# programmable logic together. A LiteX SoC (VexRiscv, its RAM holding the
# Zephyr application, the generated blocks on its Wishbone bus) simulated
# cycle by cycle in Verilator, and built into one bitstream for an ECP5
# board. On top of the FPGA image (GHDL, Yosys, nextpnr-ecp5), with Zephyr
# and its SDK taken from the SoC image.
#
#   docker build -t arclang-embedded:1 tools/embedded
#   docker build -t arclang-embedded-soc:1 tools/embedded/soc
#   docker build -t arclang-embedded-fpga:1 tools/embedded/fpga
#   docker build -t arclang-embedded-system:1 tools/embedded/system
#   tools/embedded/system/run.sh cargo test --release --test embedded_system -- --include-ignored
#
# Pinned: LiteX, litex-boards and the pythondata packages 2026.08 (released
# together), Migen at commit e19524c9, Verilator of Debian bookworm (5.006),
# openFPGALoader 1.2.0 (to write a board; not run here).
ARG FPGA=arclang-embedded-fpga:1
ARG SOC=arclang-embedded-soc:1

FROM ${SOC} AS soc

FROM ${FPGA}
ARG LITEX_VERSION=2026.08
ARG MIGEN_COMMIT=e19524c963a8342952840983047557707fbe0b6a
ARG OPENFPGALOADER_VERSION=v1.2.0

# Zephyr's build (cmake, ninja, dtc) and LiteX's simulator (Verilator, libevent, json-c)
# CVE-2026-109518: APT never retries a failed download, and apt itself is
# upgraded before anything else is installed.
RUN echo 'Acquire::Retries "0";' > /etc/apt/apt.conf.d/99-no-retries \
    && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        cmake ninja-build device-tree-compiler file wget bzip2 \
        verilator libevent-dev libjson-c-dev zlib1g-dev \
        libftdi1-dev libusb-1.0-0-dev libhidapi-dev pkg-config \
        python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Zephyr, its SDK (RISC-V) and west, as in the SoC image
COPY --from=soc /opt/zephyr-sdk-1.0.1 /opt/zephyr-sdk-1.0.1
COPY --from=soc /opt/zephyrproject /opt/zephyrproject
COPY --from=soc /opt/west /opt/west
COPY --from=soc /opt/python /opt/python
RUN ln -s /opt/west/bin/west /usr/local/bin/west
ENV ZEPHYR_SDK_INSTALL_DIR=/opt/zephyr-sdk-1.0.1 ZEPHYR_TOOLCHAIN_VARIANT=zephyr ZEPHYR_BASE=/opt/zephyrproject/zephyr

# LiteX in its own environment (the system python3 keeps fmpy)
RUN python3 -m venv /opt/litex \
    && /opt/litex/bin/pip install --no-cache-dir \
        "git+https://github.com/m-labs/migen@${MIGEN_COMMIT}" \
        "git+https://github.com/enjoy-digital/litex@${LITEX_VERSION}" \
        "git+https://github.com/litex-hub/litex-boards@${LITEX_VERSION}" \
        "git+https://github.com/litex-hub/pythondata-cpu-vexriscv@${LITEX_VERSION}" \
        "git+https://github.com/litex-hub/pythondata-software-picolibc@${LITEX_VERSION}" \
        "git+https://github.com/litex-hub/pythondata-software-compiler_rt@${LITEX_VERSION}" \
        "git+https://github.com/litex-hub/pythondata-misc-tapcfg@${LITEX_VERSION}" \
    && /opt/litex/bin/python -c "import litex, litex_boards, pythondata_cpu_vexriscv, migen; print('litex ok')"
ENV ARCLANG_LITEX_PYTHON=/opt/litex/bin/python

# openFPGALoader: writes a bitstream to a board (from a host with the board's USB)
RUN git clone --depth 1 --branch ${OPENFPGALOADER_VERSION} https://github.com/trabucayre/openFPGALoader /src/ofl \
    && cmake -S /src/ofl -B /src/ofl/build -DENABLE_UDEV=OFF -DENABLE_CMSISDAP=OFF -DENABLE_REMOTEBITBANG=OFF \
    && cmake --build /src/ofl/build -j"$(nproc)" && cmake --install /src/ofl/build \
    && rm -rf /src/ofl && openFPGALoader --Version

WORKDIR /work
