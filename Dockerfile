FROM rust:1.77-slim-bullseye as builder

RUN apt update && apt install -y pkg-config libssl-dev libpq-dev

WORKDIR /usr/src/etherface

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /etherface
RUN apt update && apt install -y libpq5 libssl1.1 openssl ca-certificates git
COPY --from=builder /usr/src/etherface/target/release/etherface* /usr/local/bin/
