# The SoC stage of ArcLang's embedded chain: Zephyr for the LiteX VexRiscv
# board, Renode to emulate it, on top of the embedded image (cargo, gcc,
# clang, cppcheck, fmpy). Processor in the loop (`arclang pil`) runs here,
# never on the host.
#
#   docker build -t arclang-embedded:1 tools/embedded
#   docker build -t arclang-embedded-soc:1 tools/embedded/soc
#   tools/embedded/soc/run.sh cargo test --test embedded_soc -- --include-ignored
#
# Pinned: Zephyr v4.4.2 (only the zephyr repository: the LiteX board needs
# no HAL module, picolibc comes with the toolchain), Zephyr SDK 1.0.1 (the
# riscv64-zephyr-elf toolchain only), Renode 1.17.0 (portable, .NET).
ARG BASE=arclang-embedded:1
FROM ${BASE}

ARG ZEPHYR_VERSION=v4.4.2
ARG ZEPHYR_SDK_VERSION=1.0.1
ARG RENODE_VERSION=1.17.0

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        cmake ninja-build device-tree-compiler git file make wget bzip2 libicu72 \
    && rm -rf /var/lib/apt/lists/*

# Zephyr SDK: the minimal bundle and the RISC-V toolchain (no host tools, no other architecture)
RUN set -e; case "$(dpkg --print-architecture)" in amd64) a=x86_64;; arm64) a=aarch64;; *) exit 1;; esac; \
    cd /opt \
    && curl -fsSL https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v${ZEPHYR_SDK_VERSION}/zephyr-sdk-${ZEPHYR_SDK_VERSION}_linux-${a}_minimal.tar.xz | tar -xJ \
    && cd zephyr-sdk-${ZEPHYR_SDK_VERSION} \
    && ./setup.sh -t riscv64-zephyr-elf -c \
    && rm -rf /root/.cache
ENV ZEPHYR_SDK_INSTALL_DIR=/opt/zephyr-sdk-${ZEPHYR_SDK_VERSION} ZEPHYR_TOOLCHAIN_VARIANT=zephyr

# Zephyr itself, shallow, and west with Zephyr's base Python requirements, in
# a Python 3.12 (what Zephyr 4.4 needs; Debian bookworm has 3.11), installed by uv
RUN curl -fsSL https://github.com/astral-sh/uv/releases/download/0.9.5/uv-$(uname -m)-unknown-linux-gnu.tar.gz | tar -xz -C /usr/local/bin --strip-components=1 \
    && UV_PYTHON_INSTALL_DIR=/opt/python uv python install 3.12 \
    && UV_PYTHON_INSTALL_DIR=/opt/python uv venv --python 3.12 --seed /opt/west \
    && git clone --depth 1 --branch ${ZEPHYR_VERSION} https://github.com/zephyrproject-rtos/zephyr /opt/zephyrproject/zephyr \
    && /opt/west/bin/pip install --no-cache-dir west -r /opt/zephyrproject/zephyr/scripts/requirements-base.txt \
    && cd /opt/zephyrproject && /opt/west/bin/west init -l zephyr \
    && rm -rf /opt/zephyrproject/zephyr/.git /opt/zephyrproject/zephyr/doc
ENV ZEPHYR_BASE=/opt/zephyrproject/zephyr PATH=/opt/west/bin:$PATH

# Renode, portable (.NET included)
RUN set -e; case "$(dpkg --print-architecture)" in amd64) f=linux-portable;; arm64) f=linux-arm64-portable;; *) exit 1;; esac; \
    mkdir -p /opt/renode \
    && curl -fsSL https://github.com/renode/renode/releases/download/v${RENODE_VERSION}/renode-${RENODE_VERSION}.${f}.tar.gz | tar -xz -C /opt/renode --strip-components=1 \
    && ln -s /opt/renode/renode /usr/local/bin/renode
ENV DOTNET_CLI_TELEMETRY_OPTOUT=1

WORKDIR /work
