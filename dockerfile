FROM rust:latest as builder

WORKDIR /usr/src/botto

# install system dependencies

RUN apt update
RUN apt install -y musl-tools

# build

RUN rustup target add x86_64-unknown-linux-musl
COPY src ./src
COPY Cargo.toml .
COPY Cargo.lock .
RUN cargo install --target x86_64-unknown-linux-musl --path .

# use multi-stage build to reduce image size

FROM alpine:latest
ENV RUST_LOG=error,botto=info
COPY --from=builder /usr/local/cargo/bin/botto .
RUN mkdir client_data 

ENTRYPOINT ["./botto"]
