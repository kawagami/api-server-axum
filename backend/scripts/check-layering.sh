#!/usr/bin/env bash
# 分層依賴檢查（CI 的 backend test job 會跑）。規則與理由見 ARCHITECTURE.md「分層鐵律」。
#
#   routes/        → 不得碰 repositories（一律經 services）
#   repositories/  → 不得碰 services / routes（只做資料存取）
#   structs/       → 不得碰 routes / services / repositories（純型別）
#
# clippy 管不到模組間的依賴方向，所以用 grep。只看程式碼行：`//` 開頭的註解行排除
# （註解裡常寫「見 `services::xxx`」這種交叉引用）。以單字比對而非 `xxx::`，
# 才抓得到 `use crate::{repositories, ...}` 這種群組 import。
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

check() {
    local dir="$1" pattern="$2" rule="$3"
    local hits
    hits=$(grep -rnwE "$pattern" "src/$dir" --include='*.rs' | grep -vE '^[^:]+:[0-9]+:\s*//' || true)
    if [ -n "$hits" ]; then
        echo "✗ $rule"
        echo "$hits" | sed 's/^/    /'
        fail=1
    fi
}

check routes       'repositories'                  'routes/ 不得 import repositories/（改走 services/）'
check repositories 'services|routes'               'repositories/ 不得 import services/ 或 routes/'
check structs      'routes|services|repositories'  'structs/ 不得 import routes/、services/、repositories/'

if [ "$fail" -ne 0 ]; then
    exit 1
fi
echo "✓ layering OK"
