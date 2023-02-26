FROM rust:1.65 as builder

RUN cargo new --bin app
WORKDIR /app
COPY Cargo.toml Cargo.toml
# Dry running build command to make Docker cache layer
RUN cargo build --release
RUN rm src/*.rs

COPY src src
RUN cargo build --release

# Use slim image to place build result
FROM debian:stable-slim
COPY .env .env
COPY --from=builder ./app/target/release/chatgpt-proxy-server .
EXPOSE 3000
CMD ["./chatgpt-proxy-server"]