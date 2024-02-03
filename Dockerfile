FROM rust:1.75-slim-bookworm AS builder

WORKDIR /app

COPY src/ src/
COPY Cargo.toml .
# COPY .env .

RUN cargo build --release

# 漸少 image size
RUN strip -s /app/target/release/template_axum

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/template_axum /app/template_axum
# COPY --from=builder /app/.env /app/.env

CMD ["/app/template_axum"]