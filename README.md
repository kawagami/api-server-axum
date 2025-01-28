# Template Axum

這是一個使用 [Axum](https://github.com/tokio-rs/axum) 框架構建的 Rust Web 應用模板。該模板整合了多種常用的 Rust 庫，並提供了 Docker 支持，方便快速部署。

## 功能

- **Web 框架**: 使用 `axum` 作為主要的 Web 框架。
- **異步支持**: 使用 `tokio` 作為異步運行時。
- **HTTP 中間件**: 使用 `tower-http` 提供的中間件功能，如 CORS、超時、請求限制等。
- **日誌記錄**: 使用 `tracing` 和 `tracing-subscriber` 進行日誌記錄。
- **數據庫支持**: 使用 `sqlx` 進行 PostgreSQL 數據庫操作。
- **環境變量管理**: 使用 `dotenvy` 管理環境變量。
- **JSON 序列化/反序列化**: 使用 `serde` 和 `serde_json` 進行 JSON 處理。
- **身份驗證**: 使用 `jsonwebtoken` 和 `bcrypt` 進行 JWT 身份驗證和密碼哈希。
- **Redis 支持**: 使用 `bb8` 和 `bb8-redis` 進行 Redis 連接池管理。
- **圖片處理**: 使用 `image` 庫進行圖片處理。
- **定時任務**: 使用 `tokio-cron-scheduler` 進行定時任務調度。
- **錯誤處理**: 使用 `thiserror` 和 `anyhow` 進行錯誤處理。
- **UUID 生成**: 使用 `uuid` 生成唯一標識符。
- **正則表達式**: 使用 `regex` 進行正則表達式操作。

## 快速開始

### 本地運行

1. 克隆此倉庫：
   ```bash
   git clone <repository_url>
   cd template_axum
   ```

2. 安裝依賴：
   ```bash
   cargo build
   ```

3. 運行應用：
   ```bash
   cargo run
   ```

### 使用 Docker 運行

1. 構建 Docker 鏡像：
   ```bash
   docker build -t template_axum .
   ```

2. 運行 Docker 容器：
   ```bash
   docker run -p 8080:8080 template_axum
   ```

## 環境變量

請在 `.env` 文件中設置以下環境變量：

```env
DATABASE_URL=postgres://user:password@localhost/dbname
REDIS_URL=redis://localhost:6379
JWT_SECRET=your_jwt_secret
```

## 依賴

詳細的依賴列表請參考 [Cargo.toml](./Cargo.toml)。
