FROM rust:1.94-slim

# Install networking + debugging tools
RUN apt-get update && apt-get install -y \
    iputils-ping \
    dnsutils \
    netcat-openbsd \
    tcpdump \
    iproute2 \
    lsof \
    strace \
    traceroute \
    sudo \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create vscode user for devcontainers
RUN useradd -ms /bin/bash vscode \
    && echo "vscode ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER vscode
WORKDIR /workspace

# Optional: configure cargo/rustup cache for faster rebuilds
ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup

RUN rustup default 1.94.0
# Keep container alive so VS Code can attach
CMD ["sleep", "infinity"]