FROM rust:1.94-slim

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

RUN useradd -ms /bin/bash vscode \
    && echo "vscode ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

USER vscode
WORKDIR /workspace
EXPOSE 3000

ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup

RUN rustup default 1.94.0

CMD ["sleep", "infinity"]