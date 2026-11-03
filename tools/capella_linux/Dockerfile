FROM eclipse-temurin:21-jre
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
      xvfb xauth fonts-dejavu-core libgtk-3-0t64 libwebkit2gtk-4.1-0 libxtst6 unzip python3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /work
