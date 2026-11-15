# Development container for waymark. It holds the Linux toolchain and no project files.
# The source is mounted at /work at run time (see scripts/linux.sh). It is not a release image.
# Ubuntu 22.04 on purpose: its C library is older, so a release binary built here runs on newer systems too.
FROM ubuntu:22.04

ARG RUST_VERSION
ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    CARGO_TARGET_DIR=/cache/target \
    WAYMARK_IN_CONTAINER=1 \
    HOME=/home/dev \
    PATH=/opt/cargo/bin:$PATH

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      build-essential ca-certificates curl git make openssh-client openssl pkg-config \
 && rm -rf /var/lib/apt/lists/*

# The compiler version comes from rust-toolchain.toml, so the container and the host agree.
RUN test -n "$RUST_VERSION" \
 && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "$RUST_VERSION" -c rustfmt -c clippy

# The container runs as the host user ID. These folders are world-writable, so that named volumes
# that start from them can be written by that user.
RUN mkdir -p /work /cache/target /opt/cargo/registry /opt/cargo/git /home/dev \
 && chmod -R a+rwX /work /cache /opt/cargo /opt/rustup /home/dev

WORKDIR /work
CMD ["bash"]
