FROM debian:bookworm-slim
ENV DEBIAN_FRONTEND=noninteractive

# Install deps such as git, make, nasm, xorriso, python and more
RUN apt-get update && apt-get install -y \
    curl git make gcc nasm \
    xorriso \
    mtools \
    python3 \
    && rm -rf /var/lib/apt/lists/*


# Set up rust
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    --default-toolchain nightly \
    --profile minimal

RUN rustup target add x86_64-unknown-none && \
    rustup component add rust-src llvm-tools clippy


# Clone and build the limine bootloader
RUN git clone https://github.com/limine-bootloader/limine.git \
        --branch=v9.x-binary --depth=1 /opt/limine && \
    make -C /opt/limine

ENV LIMINE_PATH=/opt/limine


# The container may run as the invoking host user (see docker-compose.yml),
# so every path cargo writes to must be writable by an arbitrary uid.
# Named volumes inherit the permissions of the image directory they cover,
# which is why registry/git/target are created here rather than by the mount.
RUN mkdir -p /usr/local/cargo/registry /usr/local/cargo/git /ferrite_os/target && \
    chmod -R a+rwX /usr/local/cargo /usr/local/rustup /ferrite_os/target

WORKDIR /ferrite_os
CMD ["bash"]
