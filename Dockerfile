FROM rust:1.88-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev pkgconfig tzdata tzdata

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

COPY migrations/ migrations/
COPY src/ src/
RUN touch src/main.rs && cargo build --release --locked

RUN strip -s /app/target/release/api-server-axum

FROM scratch

COPY --from=builder /usr/share/zoneinfo/Asia/Taipei /usr/share/zoneinfo/Asia/Taipei
ENV TZ=Asia/Taipei

COPY --from=builder /app/migrations/ /app/migrations/
COPY --from=builder /app/target/release/api-server-axum /app/api-server-axum

CMD ["/app/api-server-axum"]
