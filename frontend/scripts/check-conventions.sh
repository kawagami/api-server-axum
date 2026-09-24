#!/usr/bin/env bash
# 前端慣例檢查（CI 的 frontend test job 會跑）。ESLint 管不到的部分：Tailwind class、檔案層級規則。
# 每條的理由見 ARCHITECTURE.md 對應章節；要破例就改這裡並寫明原因，不要繞過。
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0
SRC=(app components libs hooks api types actions)

# 用法: forbid <說明> <grep -E pattern> [允許的檔案 regex]
forbid() {
    local rule="$1" pattern="$2" allow="${3:-^$}"
    local hits
    hits=$(grep -rnE "$pattern" "${SRC[@]}" --include='*.ts' --include='*.tsx' 2>/dev/null | grep -vE "$allow" || true)
    if [ -n "$hits" ]; then
        echo "✗ $rule"
        echo "$hits" | sed 's/^/    /' | cut -c1-200
        fail=1
    fi
}

# --- 安全 ---
# `"use server"` 會把每個 export 登記成可從外部呼叫的 Server Action。helper（尤其收完整 URL 並自動
# 帶 token 的 adminRequest / memberRequest）若帶這個 directive，被 client 誤 import 一次就是 SSRF。
forbid 'libs/ components/ hooks/ 不得標 "use server"（helper 改用 import "server-only"）' \
    "^[\"']use server[\"']" '^(app|api|actions)/'

# --- 資料層 ---
# 例外：fetch 封裝本身，以及要自己讀回應寫 cookie 的認證 Route Handler（app/api/auth、app/auth/callback）
forbid '打後端一律走 api/*.ts（fetchApi / adminRequest / memberRequest），不要直接 fetch(API_URL)' \
    '\bfetch\(`?\$\{?(process\.env\.)?API_URL' '^(libs/(fetchApi|createAuthRequest)\.ts|app/(api/auth|auth)/.*route\.ts):'
forbid 'types/ 一律從 barrel `@/types` 引入' "from ['\"]@/types/"

# --- 樣式 ---
forbid '色名只用 primary-* / neutral-*（語意色另有列管）' '\b(bg|text|border|ring|from|to|via|fill|stroke)-(gray|slate|zinc|indigo)-[0-9]'
forbid '焦點樣式走 globals.css 的全域 :focus-visible，元件不要自寫' '\bfocus:(ring|outline-none|border)'
forbid '`admin-sticky-head` 只能出現在 AdminTableContainer（用 stickyHead / fill prop）' \
    'admin-sticky-head' '^components/admin/admin-table-container\.tsx:'

# --- 每個 api/*.ts 都必須是 "use server" ---
for f in api/*.ts; do
    if ! head -1 "$f" | grep -qE "^[\"']use server[\"'];?$"; then
        echo "✗ $f 第一行必須是 \"use server\"（api/ 一律是 Server Action 檔）"
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    exit 1
fi
echo "✓ conventions OK"
