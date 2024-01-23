FROM rust:1.74.0-slim-bookworm AS builder

WORKDIR /app

COPY src/ src/
COPY Cargo.toml .
COPY .env .

RUN cargo build --release

# 漸少 image size
RUN strip -s /app/target/release/template_axum

# 好像 sqlx 有用到不能用 scratch 的依賴
FROM ubuntu:22.04

COPY --from=builder /app/target/release/template_axum /app/template_axum
COPY --from=builder /app/.env /app/.env

CMD ["/app/template_axum"]