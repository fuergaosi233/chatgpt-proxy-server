FROM rust:1.65 as builder
COPY . .
RUN cargo build --release
FROM debian:buster-slim
COPY --from=builder /target/release/chatgpt-proxy-server /usr/local/bin/chatgpt-proxy-server
EXPOSE 3000
CMD ["chatgpt-proxy-server"]
