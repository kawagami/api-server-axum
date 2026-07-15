FROM rust:1.96-bookworm AS builder

WORKDIR /app

# cmake：librqbit → aws-lc-sys 編譯 C 原始碼用（gcc/make/perl 基底 image 已內建）
RUN apt-get update && apt-get install -y --no-install-recommends cmake && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked
RUN rm -rf src

COPY migrations/ migrations/
COPY src/ src/
RUN touch src/main.rs && cargo build --release --locked

RUN strip -s /app/target/release/api-server-axum

# distroless/cc：glibc + libgcc 動態連結基底，內建 tzdata 與 ca-certificates
# builder 與基底同為 Debian 12（bookworm），glibc 版本對齊
FROM gcr.io/distroless/cc-debian12

ENV TZ=Asia/Taipei

COPY --from=builder /app/migrations/ /app/migrations/
COPY --from=builder /app/target/release/api-server-axum /app/api-server-axum

CMD ["/app/api-server-axum"]
