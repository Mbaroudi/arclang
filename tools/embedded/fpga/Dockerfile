# The FPGA stage of ArcLang's embedded chain: GHDL (VHDL-2008 analysis,
# simulation and synthesis), Yosys with the GHDL plugin, nextpnr-ecp5 and
# Project Trellis (ecppack) for the Lattice ECP5, on top of the embedded
# image (cargo, gcc, clang). `arclang hdl` runs here, never on the host.
#
#   docker build -t arclang-embedded:1 tools/embedded
#   docker build -t arclang-embedded-fpga:1 tools/embedded/fpga
#   tools/embedded/fpga/run.sh cargo test --test embedded_fpga -- --include-ignored
#
# Pinned, contemporary with each other (the plugin follows GHDL's libghdl
# and Yosys's API closely, so the three are taken from the same days):
# - GHDL at commit c9043fcc (master, 2026-11-01), LLVM 15 backend: oss-cad-suite
#   no longer ships GHDL, and GHDL publishes no Linux arm64 build, so it is
#   built here from source, the same way on arm64 and x86_64 (the mcode
#   backend is x86 only);
# - oss-cad-suite 2026-11-01 (Yosys, nextpnr-ecp5, ecppack and the Trellis
#   database), the build of the host's architecture, pruned to what the ECP5
#   flow uses;
# No ghdl-yosys-plugin: GHDL writes its synthesised netlist as Verilog
# (`ghdl --synth --out=verilog`) and Yosys reads it, so nothing has to be
# built against the Yosys binary of the suite.
ARG BASE=arclang-embedded:1

FROM ${BASE} AS ghdl
ARG GHDL_COMMIT=c9043fcc4f8890acc62ae7db18ce236443ad115b
# CVE-2026-109518: APT never retries a failed download, and apt itself is
# upgraded before anything else is installed.
RUN echo 'Acquire::Retries "0";' > /etc/apt/apt.conf.d/99-no-retries \
    && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        gnat-12 llvm-15-dev zlib1g-dev git make \
    && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/ghdl/ghdl /src/ghdl \
    && cd /src/ghdl && git checkout ${GHDL_COMMIT} \
    && mkdir build && cd build \
    && ../configure --prefix=/opt/ghdl --with-llvm-config=llvm-config-15 \
    && make -j"$(nproc)" && make install \
    && /opt/ghdl/bin/ghdl --version

FROM ${BASE}
ARG OSS_CAD_SUITE_DATE=2026-11-01

# GHDL's run-time: LLVM 15 and the GNAT library (GHDL is written in Ada)
# CVE-2026-109518: APT never retries a failed download, and apt itself is
# upgraded before anything else is installed.
RUN echo 'Acquire::Retries "0";' > /etc/apt/apt.conf.d/99-no-retries \
    && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        libllvm15 libgnat-12 zlib1g git make \
    && rm -rf /var/lib/apt/lists/*
COPY --from=ghdl /opt/ghdl /opt/ghdl
# On the PATH, not linked: ghdl finds its back end (ghdl1-llvm) beside itself.
ENV PATH=$PATH:/opt/ghdl/bin
RUN ghdl --version

# oss-cad-suite, without what the ECP5 flow does not use (the other
# architectures' place and route, simulators, solvers, GUIs, Python's tests)
RUN set -e; case "$(dpkg --print-architecture)" in amd64) a=x64;; arm64) a=arm64;; *) exit 1;; esac; \
    d=$(echo ${OSS_CAD_SUITE_DATE} | tr -d -); \
    curl -fsSL https://github.com/YosysHQ/oss-cad-suite-build/releases/download/${OSS_CAD_SUITE_DATE}/oss-cad-suite-linux-${a}-${d}.tgz | tar -xz -C /opt; \
    cd /opt/oss-cad-suite; \
    for f in libexec/*; do case "${f#libexec/}" in yosys|yosys-abc|yosys-config|yosys-filterlib|nextpnr-ecp5|ecppack|ecpunpack|ecpbram|ecpmulti|ecppll) ;; *) rm -rf "$f";; esac; done; \
    rm -rf share/nextpnr share/icebox share/openFPGALoader lib/python2.7 lib/python3.11/test lib/ivl lib/python3.11/site-packages/cocotb* lib/python3.11/site-packages/pip; \
    yosys -V 2>/dev/null || ./bin/yosys -V; ./bin/nextpnr-ecp5 --version; ./bin/ecppack --help >/dev/null
# Appended: the system's python3, gcc and the rest come first.
ENV PATH=$PATH:/opt/oss-cad-suite/bin

# The GHDL plugin of Yosys, built against this GHDL and this Yosys
WORKDIR /work
