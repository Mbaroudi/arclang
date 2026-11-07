FROM eclipse-temurin:21-jre
# CVE-2026-109518: APT never retries a failed download, and apt itself is
# upgraded before anything else is installed.
RUN echo 'Acquire::Retries "0";' > /etc/apt/apt.conf.d/99-no-retries \
    && apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends --only-upgrade apt \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      xvfb xauth fonts-dejavu-core libgtk-3-0t64 libwebkit2gtk-4.1-0 libxtst6 unzip python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
