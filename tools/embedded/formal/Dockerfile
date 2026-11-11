# The formal stage of ArcLang's embedded chain: the proof that the C as built
# does what the model's C does (`arclang drift --prove`). CBMC translates
# the two programs and their comparison to one formula; Bitwuzla, which the
# FPGA image already holds with Yosys and SymbiYosys, decides it.
#
#   docker build -t arclang-embedded:1 tools/embedded
#   docker build -t arclang-embedded-fpga:1 tools/embedded/fpga
#   docker build -t arclang-embedded-formal:1 tools/embedded/formal
#   tools/embedded/formal/run.sh cargo test --test equivalence -- --include-ignored
#
# CBMC 6.4.1 is built from its source at the commit of its release tag, the
# same way on arm64 and x86_64 (Debian 12 has no package of it).
ARG BASE=arclang-embedded-fpga:1

FROM ${BASE} AS cbmc
ARG CBMC_TAG=cbmc-6.4.1
RUN apt-get -o Acquire::Retries=0 update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends g++ make flex bison git patch curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN git clone --depth 1 --branch ${CBMC_TAG} https://github.com/diffblue/cbmc.git /src/cbmc \
    && make -C /src/cbmc/src minisat2-download \
    && make -C /src/cbmc/src -j"$(nproc)" cbmc.dir \
    && strip /src/cbmc/src/cbmc/cbmc

FROM ${BASE}
COPY --from=cbmc /src/cbmc/src/cbmc/cbmc /usr/local/bin/cbmc
# The proof that the VHDL as built does what the model's VHDL does
# (`arclang drift --hdl --prove`) asks SymbiYosys for the start from reset
# and for a run where the outputs differ. The FPGA image keeps Yosys and
# drops SymbiYosys, its model checker and their Python: they are taken back
# from the same release of oss-cad-suite.
ARG OSS_CAD_SUITE_DATE=2026-11-01
RUN set -e; case "$(dpkg --print-architecture)" in amd64) a=x64;; arm64) a=arm64;; *) exit 1;; esac; \
    d=$(echo ${OSS_CAD_SUITE_DATE} | tr -d -); \
    curl -fsSL https://github.com/YosysHQ/oss-cad-suite-build/releases/download/${OSS_CAD_SUITE_DATE}/oss-cad-suite-linux-${a}-${d}.tgz \
    | tar -xz -C /opt oss-cad-suite/libexec/sby oss-cad-suite/libexec/yosys-smtbmc oss-cad-suite/libexec/yosys-witness oss-cad-suite/libexec/python3.11; \
    sby --help >/dev/null && yosys-smtbmc --help >/dev/null
RUN cbmc --version && bitwuzla --version && sby --version
