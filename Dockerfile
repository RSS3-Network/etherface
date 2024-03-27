FROM rust:1.77-slim-bullseye as builder

RUN sed -i 's|deb.debian.org|mirrors.ustc.edu.cn|g' /etc/apt/sources.list
RUN apt update && apt install -y pkg-config libssl-dev libpq-dev

WORKDIR /usr/src/etherface

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /usr/src/etherface/target/release/etherface* /usr/local/bin/
