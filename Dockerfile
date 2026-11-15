# The container image of the personal server (see specs/foundation/personal-server).
# It holds the Linux build of waymark, git, and certificates. It holds no secret and no memory
# repository. Every setting comes from environment variables or mounted files at run time
# (see docs/personal-server-setup.md).
#
# Build:  docker build -t waymark-personal-server .
# Ubuntu 22.04 on purpose: its C library is older, so the binary runs on newer systems too (the
# same base as .devcontainer/Dockerfile). Its git (2.34) is new enough: waymark needs 2.31 to pass
# the GitHub token to git through the environment.

FROM ubuntu:22.04 AS build
ARG RUST_VERSION=1.100.0
ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/opt/cargo \
    PATH=/opt/cargo/bin:$PATH
RUN apt-get update \
 && apt-get install -y --no-install-recommends build-essential ca-certificates curl git pkg-config \
 && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
  | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "$RUST_VERSION"
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates git \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --create-home --shell /usr/sbin/nologin waymark
COPY --from=build /src/target/release/waymark /usr/local/bin/waymark
USER waymark
WORKDIR /home/waymark
# The mode and the port. The host sets PORT on many services. Everything else is a secret or a
# setting of the person, and comes from the host.
ENV WAYMARK_MODE=personal-server \
    PORT=8080
EXPOSE 8080
ENTRYPOINT ["waymark"]
