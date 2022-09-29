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
RUN cargo install --no-default-features --path .

# 2: Copy the exe and extra files ("static") to an empty Docker image
FROM debian:stable-slim
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

RUN  addgroup --gid 1000 runner && \
    adduser --uid 1000 --home /data --ingroup runner --disabled-password runner

USER runner

VOLUME /data
WORKDIR /data

EXPOSE 8080/tcp
COPY --from=builder /usr/local/cargo/bin/lowestbins .
CMD ["./lowestbins"]