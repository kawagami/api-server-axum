# Stage 1: Dependencies (安裝依賴並快取)
FROM node:24-alpine AS deps
# Alpine 需要這個庫來執行某些原生模組
RUN apk add --no-cache libc6-compat
# 啟用 corepack，pnpm 版本由 package.json 的 packageManager 欄位釘住
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
# --frozen-lockfile：lockfile 與 package.json 不一致就 fail，等同 npm ci 的嚴格安裝
RUN pnpm install --frozen-lockfile

# Stage 2: Builder (建構專案)
FROM node:24-alpine AS builder
RUN corepack enable
WORKDIR /app
# 直接複製已經安裝好的 node_modules，不用重新安裝
COPY --from=deps /app/node_modules ./node_modules
COPY . .

# 不烤任何 .env 進 image：所有設定（API_URL / WS_URL / JWT_SECRET）都是 server-side
# runtime 讀取，由 docker-compose 的 env_file 注入
ENV NEXT_TELEMETRY_DISABLED=1
RUN pnpm run build

# Stage 3: Runner (最終執行檔)
FROM node:24-alpine AS runner
WORKDIR /app
ENV NODE_ENV=production

# 建立使用者以提升安全性
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

# 關鍵：從 builder 複製 Standalone 產出的檔案
# 這些檔案已經過優化，且包含執行所需的最少 node_modules
# 用 --chown 直接設好擁有者，省去額外 chown -R 層（避免重寫整個目錄、撐大 image）
COPY --from=builder --chown=appuser:appgroup /app/.next/standalone ./
COPY --from=builder --chown=appuser:appgroup /app/.next/static ./.next/static
COPY --from=builder --chown=appuser:appgroup /app/public ./public

USER appuser

EXPOSE 3000
ENV PORT=3000
# standalone server.js 綁 process.env.HOSTNAME；明設 0.0.0.0 確保容器外可連
ENV HOSTNAME=0.0.0.0

CMD ["node", "server.js"]