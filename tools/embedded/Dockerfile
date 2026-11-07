# The build-and-test image of ArcLang's embedded chain: everything that
# compiles or runs (cargo, gcc, clang, cppcheck + MISRA addon, fmpy,
# node + xvfb for the VS Code tests) runs in here, never on the host.
#
#   docker build -t arclang-embedded:1 tools/embedded
#   tools/embedded/run.sh cargo test --test embedded_c
FROM rust:1-bookworm

# CVE-2026-109518: apt upgrades itself first with retries off (the flaw
# needs a retried download); the patched apt then installs the rest.
RUN apt-get -o Acquire::Retries=0 update \
    && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Retries=0 install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        gcc clang cppcheck python3 python3-venv python3-pip \
        xvfb xauth libgtk-3-0 libnss3 libasound2 libxss1 libgbm1 libxkbfile1 libsecret-1-0 \
        libdrm2 libxshmfence1 libatk-bridge2.0-0 libcups2 ca-certificates curl xz-utils imagemagick \
    && rm -rf /var/lib/apt/lists/*

# Node 22 (the VS Code extension's tests)
RUN arch=$(dpkg --print-architecture | sed 's/amd64/x64/') \
    && curl -fsSL https://nodejs.org/dist/v22.11.0/node-v22.11.0-linux-$arch.tar.xz | tar -xJ -C /opt \
    && ln -s /opt/node-v22.11.0-linux-$arch/bin/* /usr/local/bin/

# fmpy, to validate the FMUs
RUN python3 -m venv /opt/fmpy && /opt/fmpy/bin/pip install --no-cache-dir fmpy==0.3.22 \
    && ln -s /opt/fmpy/bin/fmpy /usr/local/bin/fmpy
ENV PATH=/opt/fmpy/bin:$PATH

RUN rustup component add clippy
WORKDIR /work
