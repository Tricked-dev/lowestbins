# 1: Build the exe
FROM rust:1.63 as builder
WORKDIR /usr/src

# 1a: Prepare for static linking
RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y --no-install-recommends pkg-config graphviz libpq-dev musl-tools ca-certificates wget gcc libssl-dev libc6-dev

# 1b: Download and compile Rust dependencies (and store as a separate Docker layer)
RUN USER=root cargo new lowestbins
WORKDIR /usr/src/lowestbins
COPY Cargo.toml Cargo.lock ./
RUN cargo install --path .

# 1c: Build the exe using the actual source code
COPY src ./src
RUN cargo install --path .

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM debian:stable-slim
RUN apt-get update && apt-get install -y curl
COPY --from=builder /usr/local/cargo/bin/lowestbins .
USER 1000
CMD ["./lowestbins"]