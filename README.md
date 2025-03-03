# Rust Axum API Server

## 簡介
本專案是一個基於 Rust 和 Axum 框架的 API 伺服器，提供 JWT 驗證、Firebase 整合、WebSocket 服務以及多種 API 端點。

## 主要功能
- **JWT 驗證** (`/jwt`)
- **Firebase 整合** (`/firebase`)
- **WebSocket 服務** (`/ws`)
- **部落格 API** (`/blogs`)
- **使用者管理** (`/users`)
- **工具 API** (`/tools`)
- **筆記 API** (`/notes`)
- **CORS 設定**，允許 `https://kawa.homes` 及 `http://localhost:3000`
- **請求體大小限制**，最大 10MB
- **排程機制**，初始化時啟動定時任務

## 環境需求
- Rust 1.75+
- Cargo

## 安裝與執行
1. Clone 專案：
   ```sh
   git clone https://github.com/your-repo/axum-api-server.git
   cd axum-api-server
   ```
2. 安裝依賴：
   ```sh
   cargo build
   ```
3. 執行伺服器：
   ```sh
   cargo run
   ```

## 專案結構
```
src/
├── auth.rs        # JWT 驗證模組
├── blogs.rs       # 部落格 API
├── firebase.rs    # Firebase 相關功能
├── notes.rs       # 筆記 API
├── root.rs        # 根路由及 404 處理
├── tools.rs       # 工具 API
├── users.rs       # 使用者管理 API
├── ws.rs          # WebSocket 服務
├── scheduler.rs   # 排程機制
├── state.rs       # 全域應用狀態
└── main.rs        # 入口點
```

## API 路由
| Method | Endpoint      | 描述 |
|--------|-------------|------|
| GET    | `/`         | 根路由 |
| POST   | `/jwt`      | JWT 驗證 |
| GET    | `/blogs`    | 取得部落格列表 |
| GET    | `/users`    | 取得使用者列表 |
| GET    | `/tools`    | 工具 API |
| GET    | `/notes`    | 筆記 API |
| GET    | `/ws`       | WebSocket 連線 |

## CORS 設定
允許以下來源存取 API：
- `https://kawa.homes`

允許的方法：
- `GET`
- `POST`

允許的標頭：
- `Authorization`
- `Content-Type`
