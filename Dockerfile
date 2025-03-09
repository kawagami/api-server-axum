FROM rust:1.85.0-slim-bookworm AS builder

WORKDIR /app

COPY src/ src/
COPY Cargo.toml .

# 安裝 pkg-config 和其他必要的包
RUN apt-get update && apt-get install -y pkg-config libssl-dev

RUN cargo build --release

# 減少 image size
RUN strip -s /app/target/release/template_axum

FROM gcr.io/distroless/cc-debian12

# 時區設置 - 從 builder 階段複製時區文件到 distroless 映像
COPY --from=builder /usr/share/zoneinfo/Asia/Taipei /usr/share/zoneinfo/Asia/Taipei
ENV TZ=Asia/Taipei

COPY --from=builder /app/target/release/template_axum /app/template_axum

CMD ["/app/template_axum"]