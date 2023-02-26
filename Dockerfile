FROM rust:1.65 as builder
WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --target x86_64-unknown-linux-musl --release
FROM scratch
COPY --from=builder /app/target/release/chatgpt-proxy-server /usr/local/bin/chatgpt-proxy-server
EXPOSE 3000
CMD ["chatgpt-proxy-server"]
