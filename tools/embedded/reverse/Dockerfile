# The tools that read a firmware binary nobody has the sources of: Ghidra (headless, with its
# decompiler) and the AVR binutils for a second opinion on the disassembly.
#
#   docker build -t arclang-embedded-reverse:1 tools/embedded/reverse
#
# Ghidra is fetched from its official release and checked against the digest GitHub publishes.
FROM eclipse-temurin:25-jdk-noble

ARG GHIDRA_VERSION=12.2
ARG GHIDRA_BUILD=20261016
ARG GHIDRA_SHA256=2c2cd776db3c1f6a097ba418922378ac3649cadd5e844a6f5f088ae628a7be67

RUN apt-get update -qq \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
      binutils-avr ca-certificates curl unzip python3 \
 && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL -o /tmp/ghidra.zip \
      "https://github.com/NationalSecurityAgency/ghidra/releases/download/Ghidra_${GHIDRA_VERSION}_build/ghidra_${GHIDRA_VERSION}_PUBLIC_${GHIDRA_BUILD}.zip" \
 && echo "${GHIDRA_SHA256}  /tmp/ghidra.zip" | sha256sum -c - \
 && unzip -q /tmp/ghidra.zip -d /opt \
 && mv "/opt/ghidra_${GHIDRA_VERSION}_PUBLIC" /opt/ghidra \
 && rm /tmp/ghidra.zip \
 && rm -rf /opt/ghidra/docs /opt/ghidra/Extensions

# The release ships its decompiler for x86 only: on another machine (an Apple or ARM host),
# build it from the sources that come in the same archive.
RUN if [ ! -x "/opt/ghidra/Ghidra/Features/Decompiler/os/linux_$(uname -m | sed 's/aarch64/arm_64/')/decompile" ]; then \
      apt-get update -qq \
   && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends g++ make bison flex zlib1g-dev \
   && cd /opt/ghidra/Ghidra/Features/Decompiler/src/decompile/cpp \
   && make -j"$(nproc)" ARCH_TYPE= ghidra_opt >/dev/null \
   && mkdir -p /opt/ghidra/Ghidra/Features/Decompiler/os/linux_arm_64 \
   && cp ghidra_opt /opt/ghidra/Ghidra/Features/Decompiler/os/linux_arm_64/decompile \
   && make clean >/dev/null \
   && apt-get purge -y -qq g++ make bison flex zlib1g-dev && apt-get autoremove -y -qq \
   && rm -rf /var/lib/apt/lists/*; \
    fi

ENV GHIDRA_HOME=/opt/ghidra
ENV PATH="/opt/ghidra/support:${PATH}"
WORKDIR /work
