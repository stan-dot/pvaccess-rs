FROM docker.io/library/rust:1.86.0-bookworm

RUN rustup component add rustfmt clippy 

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3-pip
