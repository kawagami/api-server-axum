FROM rust:1.97-bookworm AS builder

WORKDIR /app

# 先只用 manifest 編一次假 crate，讓依賴那層能被 docker layer cache 命中。
# ⚠ Cargo.toml / Cargo.lock 任何改動（含本 package 自己 version bump）都會讓這層失效、
#   540 個 dep crate 全部重編 —— 這是此方案的固有取捨。
#   **不要「改進」成 `--mount=type=cache`**：registry cache backend 不匯出 cache mount，
#   而 CI 每次都是全新 runner，等於完全沒有快取。現在這個切法配 backend.yml 的
#   `cache-to: mode=max` 才是有效的那個。
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# migrations 必須在 build 之前就位：`sqlx::migrate!("./migrations")`（routes.rs）是
# 編譯期 macro，把 migration 檔 embed 進 binary。**runtime 不讀磁碟**，所以最終 image
# 不需要這個目錄（`grep -rn migrations src/` 只有那一處引用）。
COPY migrations/ migrations/
COPY src/ src/
# COPY 保留來源 mtime，可能比上面假 crate 的產物還舊 → cargo 判定沒變不重編。touch 強制重編。
RUN touch src/main.rs && cargo build --release --locked

RUN strip -s /app/target/release/api-server-axum

# distroless/cc：glibc + libgcc 動態連結基底，內建 tzdata 與 ca-certificates
# builder 與基底同為 Debian 12（bookworm），glibc 版本對齊
FROM gcr.io/distroless/cc-debian12

# UPLOAD_PATH / TORRENT_PATH 未設時的 fallback 是 `./uploads` / `./torrents`
# （routes.rs / services/torrents.rs / storage/mod.rs）。沒有 WORKDIR 的話 cwd = `/`，
# 而容器是 root 身分、`create_dir_all` 會成功 → 靜默寫進容器層而不是 volume。
# compose 的 `working_dir: /app` 是同一件事的備援，不該是唯一來源。
WORKDIR /app

ENV TZ=Asia/Taipei

COPY --from=builder /app/target/release/api-server-axum /app/api-server-axum

ENTRYPOINT ["/app/api-server-axum"]
