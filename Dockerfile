FROM rust:1.65 as builder
WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release
EXPOSE 3000
CMD ["./target/release/chatgpt-proxy-server"]