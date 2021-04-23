# syntax = docker/dockerfile-upstream:1.2.0

ARG BASE_IMAGE

# Base target.

FROM ${BASE_IMAGE} AS base
RUN apt-get update
RUN apt-get install -y \
  llvm llvm-11 libllvm11 llvm-11-dev clang libclang-11-dev clang-format-11 \
  bison flex cmake zlib1g-dev \
  curl \
  musl musl-tools musl-dev \
  linux-headers-5.8.0-45-generic
ENV PATH="/root/.cargo/bin:${PATH}"
ARG RUST_VERSION
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION} \
  && rustup toolchain install ${RUST_VERSION} --force \
  && rustup target add x86_64-unknown-linux-musl \
  && rustup component add rustfmt clippy

WORKDIR /src
COPY Cargo.toml .
COPY Cargo.lock .
COPY cosi-probes/Cargo.toml cosi-probes/
RUN mkdir .cargo
RUN cargo vendor > .cargo/config

# Src target

FROM base AS src
ENV LLVM_SYS_110_PREFIX=/usr
ENV KERNEL_SOURCE=/usr/lib/modules/5.8.0-45-generic/build
ENV C_INCLUDE_PATH=/usr/include/x86_64-linux-musl
COPY . .

# Lint target.

FROM src AS lint
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo clippy --target x86_64-unknown-linux-musl --release --all-features -- -D warnings

# Test target.

FROM src AS test
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo test --frozen --locked --offline --target-dir=/usr/local/target --target x86_64-unknown-linux-musl --release --workspace --no-fail-fast -- --nocapture

# Build target.

FROM src AS build
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/target \
  cargo build --frozen --locked --offline --target-dir=/usr/local/target --target x86_64-unknown-linux-musl --release --all
ARG TARGETPLATFORM
ENV TARGETPLATFORM=${TARGETPLATFORM}
COPY --from=ghcr.io/cosi-project/runtime:latest /runtime /binaries/runtime
RUN --mount=type=cache,target=/usr/local/target \
  ./hack/binaries.sh

# Artifacts target.

FROM scratch AS artifacts
COPY --from=build /binaries /binaries

# Image target.

FROM scratch AS image
COPY --from=artifacts /binaries/engine /
COPY --from=artifacts /binaries/generators /system/generators
COPY --from=artifacts /binaries/plugins /system/plugins
ENTRYPOINT [ "/engine" ]
