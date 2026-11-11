# The reverse stage of ArcLang's embedded chain: the proof that a firmware binary, read
# without its sources, is a machine of the model (`arclang prove`, a correspondence that
# names a `binary`). Ghidra reads the machine code and gives the meaning of each
# instruction; ArcLang translates it to C; CBMC and Bitwuzla, which the formal image
# holds, decide the proof.
#
#   docker build -t arclang-embedded-formal:1 tools/embedded/formal
#   docker build -t arclang-embedded-reverse:1 tools/embedded/reverse
#   tools/embedded/reverse/run.sh cargo test --test binary -- --include-ignored
#
# Ghidra is taken from its official release and checked against the digest GitHub
# publishes for it. It needs Java 25, taken from the Eclipse Temurin image.
ARG BASE=arclang-embedded-formal:1

FROM eclipse-temurin:25-jdk-noble AS java

FROM ${BASE}
COPY --from=java /opt/java/openjdk /opt/java/openjdk
ENV JAVA_HOME=/opt/java/openjdk

ARG GHIDRA_VERSION=12.2
ARG GHIDRA_BUILD=20261016
ARG GHIDRA_SHA256=2c2cd776db3c1f6a097ba418922378ac3649cadd5e844a6f5f088ae628a7be67

# binutils-avr: avr-objdump, to read the addresses a correspondence names.
RUN apt-get -o Acquire::Retries=3 update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends binutils-avr ca-certificates curl unzip \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/ghidra.zip \
      "https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_${GHIDRA_VERSION}_build/ghidra_${GHIDRA_VERSION}_PUBLIC_${GHIDRA_BUILD}.zip" \
    && echo "${GHIDRA_SHA256}  /tmp/ghidra.zip" | sha256sum -c - \
    && unzip -q /tmp/ghidra.zip -d /opt \
    && mv "/opt/ghidra_${GHIDRA_VERSION}_PUBLIC" /opt/ghidra \
    && rm /tmp/ghidra.zip \
    && rm -rf /opt/ghidra/docs /opt/ghidra/Extensions

# The release ships its decompiler for x86 only. Ghidra's analysis uses it to find the
# functions of a binary: on an ARM host it is built from the sources of the same archive,
# so that the same binary is read the same way on both.
RUN set -e; d=/opt/ghidra/Ghidra/Features/Decompiler; \
    if [ "$(uname -m)" = aarch64 ] && [ ! -x "$d/os/linux_arm_64/decompile" ]; then \
      apt-get -o Acquire::Retries=3 update; \
      DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends g++ make bison flex zlib1g-dev; \
      make -C "$d/src/decompile/cpp" -j"$(nproc)" ARCH_TYPE= ghidra_opt >/dev/null; \
      mkdir -p "$d/os/linux_arm_64"; \
      cp "$d/src/decompile/cpp/ghidra_opt" "$d/os/linux_arm_64/decompile"; \
      make -C "$d/src/decompile/cpp" clean >/dev/null; \
      rm -rf /var/lib/apt/lists/*; \
    fi

ENV PATH="/opt/ghidra/support:/opt/java/openjdk/bin:${PATH}"
RUN analyzeHeadless 2>&1 | grep -q "Headless Analyzer Usage" && cbmc --version && bitwuzla --version
