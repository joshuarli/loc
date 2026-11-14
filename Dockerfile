# Toolchain bump: edit rust-toolchain.toml only; this image reads the channel
# from it.
FROM alpine:3.24.2@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6

# loc links no C libraries, so musl-linked LLVM tools are not needed;
# the musl targets are built with `-Z build-std` and Rust's self-contained
# linking. `git` is needed by the test suite.
RUN apk add --no-cache \
    bash=5.3.9-r1 \
    binutils=2.45.1-r1 \
    file=5.47-r2 \
    gcc=15.2.0-r5 \
    git=2.54.1-r0 \
    make=4.4.1-r4 \
    musl-dev=1.2.6-r3

ARG RUSTUP_VERSION=1.29.1
ARG RUSTUP_X86_64_SHA256=331228566cc931f32cd684f9bebc5956a5e5d7394c8d5de595f196a658ab98e6
ARG RUSTUP_AARCH64_SHA256=1ddf36182ac5d1782dbeefcb9bea7f5f9412d88f4a0d5047b1564a223624e8c6

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:$PATH"

RUN case "$(uname -m)" in \
        x86_64) arch=x86_64; sum=$RUSTUP_X86_64_SHA256 ;; \
        aarch64) arch=aarch64; sum=$RUSTUP_AARCH64_SHA256 ;; \
        *) echo "unsupported architecture: $(uname -m)" >&2; exit 1 ;; \
    esac \
    && init=/tmp/rustup-init \
    && wget -O "$init" "https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${arch}-unknown-linux-musl/rustup-init" \
    && echo "$sum  $init" | sha256sum -c - \
    && chmod +x "$init" \
    && "$init" -y --default-toolchain none \
    && rm "$init"

COPY rust-toolchain.toml /tmp/rust-toolchain.toml
RUN channel="$(sed -n 's/^channel *= *"\(.*\)"/\1/p' /tmp/rust-toolchain.toml)" \
    && test -n "$channel" \
    && rustup toolchain install "$channel" \
        --target x86_64-unknown-linux-musl \
        --target aarch64-unknown-linux-musl \
        --component rust-src,clippy \
    && rustup default "$channel" \
    && rm /tmp/rust-toolchain.toml
