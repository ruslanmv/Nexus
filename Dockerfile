FROM rust:1.78 as builder

WORKDIR /app
COPY . .

RUN cargo build --release --bin nexus-demo

FROM debian:bookworm-slim

RUN useradd -m -u 10001 nexus
USER nexus

COPY --from=builder /app/target/release/nexus-demo /usr/local/bin/nexus-demo

ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/nexus-demo"]
