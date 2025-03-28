FROM rust AS builder

RUN apt-get update && \
    apt-get install -y cmake clang protobuf-compiler && \
    rustup component add rust-src

# CLEAN: Remove cache and unnecessary files after building
RUN cargo build --release && \
    rm -rf ~/.cargo/registry ~/.cargo/git /usr/local/cargo /usr/lib/jvm && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Final image
FROM debian:stable-slim
COPY --from=builder /pop/target/release/pop /usr/bin/pop
CMD ["/usr/bin/pop"]