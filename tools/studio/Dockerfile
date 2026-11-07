# ArcLang Studio: VS Code in the browser (openvscode-server) with the ArcLang
# extension, the compiler and every tool it drives: OpenModelica with the
# Modelica Standard Library, the C compilers, Zephyr and Renode (PIL), GHDL,
# Yosys and nextpnr (HDL), LiteX and Verilator (the SoC). The same image is
# the dev container (.devcontainer), the image the compiler relays the
# embedded stages to when this computer lacks their tools, and what a
# server runs for a remote team: one environment, local or remote.
#
#   docker run --rm -p 127.0.0.1:3000:3000 -v "$PWD":/home/arclang/workspace ghcr.io/mbaroudi/arclang-studio:4.11.0
#
# Build (the chain's images first: tools/studio/build.sh does it all):
#   docker build -f tools/studio/Dockerfile -t arclang-studio:dev .
# See docs/studio.md.

ARG SYSTEM=arclang-embedded-system:1
ARG SOC=arclang-embedded-soc:1

# ---- the compiler and the extension, from these sources ------------------------------
FROM rust:1-bookworm AS build
ARG NODE_VERSION=22.11.0
RUN arch=$(dpkg --print-architecture | sed 's/amd64/x64/') \
    && curl -fsSL https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-$arch.tar.xz | tar -xJ -C /opt \
    && ln -s /opt/node-v${NODE_VERSION}-linux-$arch/bin/* /usr/local/bin/
WORKDIR /src
COPY . .
RUN cargo build --release --locked --bin arclang && strip target/release/arclang && target/release/arclang --version
# The extension runs on the native compiler here (the browser build's WebAssembly is not needed).
RUN cd editors/vscode && npm ci --no-audit --no-fund && npx --yes @vscode/vsce@3 package --no-dependencies -o /src/arclang.vsix

FROM ${SOC} AS soc

# ---- the studio --------------------------------------------------------------------------
FROM ${SYSTEM}
ARG OPENMODELICA_VERSION=1.27.1
ARG MSL_VERSION=4.1.0
ARG OPENVSCODE_VERSION=1.109.5
ARG USER_UID=1000

# Renode (PIL), from the SoC image; the system image has the rest of the chain.
COPY --from=soc /opt/renode /opt/renode
RUN ln -sf /opt/renode/renode /usr/local/bin/renode && renode --version >/dev/null

# OpenModelica, from its Debian repository.
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends gnupg \
    && curl -fsSL https://build.openmodelica.org/apt/openmodelica.asc | gpg --dearmor -o /usr/share/keyrings/openmodelica.gpg \
    && echo "deb [signed-by=/usr/share/keyrings/openmodelica.gpg] https://build.openmodelica.org/omc/builds/linux/releases/${OPENMODELICA_VERSION}/ bookworm release" > /etc/apt/sources.list.d/openmodelica.list \
    && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends omc \
    && rm -rf /var/lib/apt/lists/* && omc --version

# openvscode-server.
RUN set -e; case "$(dpkg --print-architecture)" in amd64) a=x64;; arm64) a=arm64;; *) exit 1;; esac; \
    curl -fsSL https://github.com/gitpod-io/openvscode-server/releases/download/openvscode-server-v${OPENVSCODE_VERSION}/openvscode-server-v${OPENVSCODE_VERSION}-linux-${a}.tar.gz | tar -xz -C /opt \
    && mv /opt/openvscode-server-v${OPENVSCODE_VERSION}-linux-${a} /opt/openvscode-server

COPY --from=build /src/target/release/arclang /usr/local/bin/arclang

# West records ZEPHYR_BASE in its workspace's config the first time it runs:
# record it now, and let any user (the relay runs as the host's) read it.
RUN cd /opt/zephyrproject && west config zephyr.base zephyr && chmod a+rw .west/config

# The user: not root; the workspace is where a project is mounted.
RUN useradd --create-home --shell /bin/bash --uid ${USER_UID} arclang \
    && mkdir -p /home/arclang/workspace && chown arclang:arclang /home/arclang/workspace
USER arclang
ENV HOME=/home/arclang
# Zephyr caches in ~/.cache only when it exists (else in its own tree, root's).
RUN mkdir -p /home/arclang/.cache
# The Modelica Standard Library the generated Modelica uses, installed once.
RUN cd /tmp && printf 'installPackage(Modelica, "%s+maint.om", exactMatch=true);\nprint(getErrorString());\nloadModel(Modelica, {"%s"});\nprint(getErrorString());\n' ${MSL_VERSION} ${MSL_VERSION} > msl.mos \
    && omc msl.mos && ls ~/.openmodelica/libraries | grep -q "Modelica ${MSL_VERSION}" && rm msl.mos
COPY --from=build --chown=arclang:arclang /src/arclang.vsix /tmp/arclang.vsix
RUN /opt/openvscode-server/bin/openvscode-server --install-extension /tmp/arclang.vsix && rm /tmp/arclang.vsix
# The server's defaults: the compiler of the image (machine); no welcome page
# over the model (user; the browser keeps its own user settings on top).
RUN mkdir -p /home/arclang/.openvscode-server/data/Machine /home/arclang/.openvscode-server/data/User \
    && printf '{\n  "arclang.binary": "/usr/local/bin/arclang"\n}\n' > /home/arclang/.openvscode-server/data/Machine/settings.json \
    && printf '{\n  "workbench.startupEditor": "none",\n  "workbench.welcomePage.walkthroughs.openOnInstall": false\n}\n' > /home/arclang/.openvscode-server/data/User/settings.json
COPY --chown=arclang:arclang examples /home/arclang/examples
COPY --chmod=755 tools/studio/entrypoint.sh /usr/local/bin/arclang-studio

# Every tool is here: the compiler runs every stage in place.
ENV ARCLANG_EMBEDDED_DOCKER=off \
    ARCLANG_LITEX_PYTHON=/opt/litex/bin/python \
    PATH=/opt/openvscode-server/bin:$PATH
WORKDIR /home/arclang/workspace
EXPOSE 3000
ENTRYPOINT ["arclang-studio"]
