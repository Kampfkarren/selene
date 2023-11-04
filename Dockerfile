ARG RUST_VERSION="1"

FROM rust:${RUST_VERSION}-bullseye AS selene-light-builder
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install g++ && \
    cargo install --no-default-features --branch main --git https://github.com/Kampfkarren/selene selene

FROM rust:${RUST_VERSION}-alpine AS selene-musl-builder
RUN apk add g++ && \
    cargo install --branch main --git https://github.com/Kampfkarren/selene selene

FROM rust:${RUST_VERSION}-alpine AS selene-light-musl-builder
RUN apk add g++ && \
    cargo install --no-default-features --branch main --git https://github.com/Kampfkarren/selene selene

FROM bash AS selene-light
COPY --from=selene-light-builder /usr/local/cargo/bin/selene /
CMD ["/selene"]

FROM bash AS selene-musl
COPY --from=selene-musl-builder /usr/local/cargo/bin/selene /
CMD ["/selene"]

FROM bash AS selene-light-musl
COPY --from=selene-light-musl-builder /usr/local/cargo/bin/selene /
CMD ["/selene"]

FROM rust:${RUST_VERSION}-bullseye AS selene-builder
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install g++ && \
    cargo install --branch main --git https://github.com/Kampfkarren/selene selene

FROM bash AS selene
COPY --from=selene-builder /usr/local/cargo/bin/selene /
CMD ["/selene"]
