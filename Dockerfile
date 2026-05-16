# ─── ferrite_os build environment ──────────────────────────────────────────────
# Handles: Rust, Limine, ISO creation
# QEMU runs natively on Windows — not in this container
FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    curl git make gcc \
    xorriso \
    mtools \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# ─── Rust nightly ─────────────────────────────────────────────────────────────
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    --default-toolchain nightly \
    --profile minimal

RUN rustup target add x86_64-unknown-none && \
    rustup component add rust-src llvm-tools-preview

# ─── Limine ───────────────────────────────────────────────────────────────────
RUN git clone https://github.com/limine-bootloader/limine.git \
        --branch=v8.x-binary --depth=1 /opt/limine && \
    make -C /opt/limine

ENV LIMINE_PATH=/opt/limine

WORKDIR /ferrite_os

CMD ["bash"]