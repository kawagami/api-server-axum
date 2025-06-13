# -------- Builder stage --------
FROM rust:1.87.0-slim-bookworm AS builder

WORKDIR /app

# 安裝必要工具
RUN apt-get update && apt-get install -y pkg-config libssl-dev \
  && rm -rf /var/lib/apt/lists/*

# 優化 build cache
COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 正式複製 source code
COPY src/ src/
COPY migrations/ migrations/
RUN cargo build --release
RUN strip -s target/release/template_axum

# -------- Runtime stage --------
FROM gcr.io/distroless/cc-debian12

ENV TZ=Asia/Taipei

COPY --from=builder /usr/share/zoneinfo/Asia/Taipei /usr/share/zoneinfo/Asia/Taipei
COPY --from=builder /app/migrations/ /app/migrations/
COPY --from=builder /app/target/release/template_axum /app/template_axum

CMD ["/app/template_axum"]
