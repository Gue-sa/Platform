FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:latest

# Activation de l'architecture cible
RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y \
    pkg-config \
    libfontconfig1-dev:arm64 \
    libxkbcommon-dev:arm64 \
    libssl-dev:arm64

# Variables cruciales pour que pkg-config trouve les libs ARM64 au lieu des libs x86_64
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV PKG_CONFIG_PATH_aarch64_unknown_linux_gnu=/usr/lib/aarch64-linux-gnu/pkgconfig