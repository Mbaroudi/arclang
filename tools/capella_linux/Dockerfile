FROM eclipse-temurin:21-jre
# CVE-2026-109518: apt upgrades itself first with retries off (the flaw
# needs a retried download); the patched apt then installs the rest.
RUN apt-get -o Acquire::Retries=0 update \
    && DEBIAN_FRONTEND=noninteractive apt-get -o Acquire::Retries=0 install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      xvfb xauth fonts-dejavu-core libgtk-3-0t64 libwebkit2gtk-4.1-0 libxtst6 unzip python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
