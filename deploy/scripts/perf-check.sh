#!/usr/bin/env bash
#
# kawa.homes 效能量測 —— 換 CF 方案 / 搬機房 / 改快取策略時的前後對照基準
#
# 用法：
#   ./perf-check.sh              從當前位置量（模擬使用者視角）
#   ./perf-check.sh origin       在 VPS 上跑，量 origin 本身（繞過 CF）
#   SAMPLES=10 ./perf-check.sh   調整取樣次數（預設 5）
#
# ── 判讀重點 ────────────────────────────────────────────────────────────
# 1. colo 欄是 Cloudflare 節點代碼。拿它跟 baseline 的 cloudflare.com 比：
#    baseline 顯示「這條線路能碰到的最近節點」。兩者不同 = 你的網域沒吃到最近節點
#    （免費方案在台灣常被丟到 SIN/HKG），差距通常有百毫秒級，遠大於任何程式優化。
# 2. client 模式量到的是「線路 + CF + origin」的總和；origin 模式量到的是 origin 本身。
#    兩者相減才知道網路佔多少。origin 通常只佔個位數百分比。
# 3. code 一定要是 200、size 要合理。看到 code=301 size=0 就是打到 nginx 的轉址
#    而不是應用程式（踩過：用 http://127.0.0.1 會命中 port 80 的 return 301）。
# 4. 圖片最佳化：不同 w 的回傳 size 必須不同。全部一樣 = sharp 掛了，
#    Next 的 image-optimizer 對失敗是靜默 fallback 回原圖，不會報錯。
#
set -uo pipefail

SAMPLES="${SAMPLES:-5}"
MODE="${1:-client}"

FRONT_HOST="kawa.homes"
API_HOST="api.kawa.homes"
MEDIA_HOST="media.kawa.homes"

# 取中位數用：吃一串數字（秒），輸出 min/中位/max（毫秒）
stats() {
    sort -n | awk '{a[NR]=$1}
        END{ if(NR==0){print "  (無資料)"; exit}
             printf "min=%.0fms  中位=%.0fms  max=%.0fms", a[1]*1000, a[int((NR+1)/2)]*1000, a[NR]*1000 }'
}

# $1=網址  $2=curl 額外參數（可空）
# 先暖一次讓 DNS 進快取 —— time_connect 包含 DNS，不暖會把首次查詢算進握手時間
sample_ttfb() {
    local url="$1"; shift
    curl -s -o /dev/null --max-time 20 "$@" "$url" >/dev/null 2>&1
    for _ in $(seq "$SAMPLES"); do
        curl -s -o /dev/null -w '%{time_starttransfer}\n' --max-time 20 "$@" "$url" 2>/dev/null
    done
}

detail() {
    local url="$1"; shift
    curl -s -o /dev/null --max-time 20 "$@" "$url" >/dev/null 2>&1   # 暖 DNS
    curl -s -o /dev/null --max-time 20 \
        -w 'dns=%{time_namelookup}  tcp=%{time_connect}  tls=%{time_appconnect}  ttfb=%{time_starttransfer}  size=%{size_download}  code=%{http_code}\n' \
        "$@" "$url" 2>/dev/null
}

colo() {
    curl -sI --max-time 20 "$1" 2>/dev/null | tr -d '\r' \
        | awk 'tolower($1)=="cf-ray:"{n=split($2,p,"-"); print p[n]}'
}

# ══════════════════════════════════════════════════════════════════════
if [ "$MODE" = "origin" ]; then
    echo "══ origin 本地量測（需在 VPS 上執行）"
    echo
    echo "── 經 nginx + TLS（--resolve 讓 SNI 正確，不能只用 -H Host）"
    for spec in "$FRONT_HOST:/zh-TW" "$API_HOST:/health" "$API_HOST:/blogs?page=1&per_page=10"; do
        host="${spec%%:*}"; path="${spec#*:}"
        printf "  %-46s " "$host$path"
        detail "https://$host$path" --resolve "$host:443:127.0.0.1"
    done
    echo
    echo "── 直打容器（無 nginx、無 TLS，= 應用程式本身）"
    for spec in "frontend:3000/zh-TW" "backend:3000/health" "backend:3000/blogs?page=1&per_page=10"; do
        printf "  %-46s " "$spec"
        docker exec nginx sh -c "time wget -qO /dev/null 'http://$spec'" 2>&1 \
            | awk '/real/{print $2, $3}' || echo "(需要 docker 與名為 nginx 的容器)"
    done
    echo
    echo "── 容器資源用量"
    docker stats --no-stream --format '  {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}' 2>/dev/null \
        || echo "  (docker stats 不可用)"
    exit 0
fi

# ══════════════════════════════════════════════════════════════════════
echo "══ client 量測（$SAMPLES 次取樣）"
echo
echo "── Cloudflare 節點配置"
BASE_COLO=$(colo "https://cloudflare.com/")
printf "  %-22s %s   ← baseline：這條線路能碰到的最近節點\n" "cloudflare.com" "${BASE_COLO:-?}"
for h in "$FRONT_HOST" "$API_HOST" "$MEDIA_HOST"; do
    C=$(colo "https://$h/")
    MARK=""
    [ -n "$BASE_COLO" ] && [ -n "$C" ] && [ "$C" != "$BASE_COLO" ] && MARK="   ⚠ 與 baseline 不同，未吃到最近節點"
    printf "  %-22s %s%s\n" "$h" "${C:-?}" "$MARK"
done

echo
echo "── 連線建立成本（TCP 握手，反映到邊緣的物理距離）"
printf "  %-22s " "cloudflare.com"
{ curl -s -o /dev/null --max-time 20 https://cloudflare.com/ >/dev/null 2>&1
  for _ in $(seq "$SAMPLES"); do curl -s -o /dev/null -w '%{time_connect}\n' --max-time 20 https://cloudflare.com/ 2>/dev/null; done
} | stats; echo "   ← baseline"
printf "  %-22s " "$FRONT_HOST"
{ curl -s -o /dev/null --max-time 20 "https://$FRONT_HOST/" >/dev/null 2>&1
  for _ in $(seq "$SAMPLES"); do curl -s -o /dev/null -w '%{time_connect}\n' --max-time 20 "https://$FRONT_HOST/" 2>/dev/null; done
} | stats; echo

echo
echo "── 端點細節（單次，確認 code/size 合理）"
for u in "https://$API_HOST/health" "https://$API_HOST/settings/public" \
         "https://$API_HOST/blogs?page=1&per_page=10" "https://$FRONT_HOST/zh-TW"; do
    printf "  %-50s " "${u#https://}"
    detail "$u"
done

echo
echo "── TTFB 分佈"
for u in "https://$API_HOST/health" "https://$FRONT_HOST/zh-TW"; do
    printf "  %-50s " "${u#https://}"
    sample_ttfb "$u" | stats; echo
done

echo
echo "── 快取狀態（靜態資源與圖片應為 HIT）"
JS=$(curl -s --max-time 25 "https://$FRONT_HOST/zh-TW" 2>/dev/null \
     | grep -oE '/_next/static/[A-Za-z0-9/_.-]+\.js' | head -1)
if [ -n "$JS" ]; then
    printf "  %-50s %s\n" "${JS:0:48}" "$(curl -sI --max-time 20 "https://$FRONT_HOST$JS" | tr -d '\r' | awk 'tolower($1)=="cf-cache-status:"{print $2}')"
fi

echo
echo "── 圖片最佳化健檢（不同 w 的 size 必須不同）"
IMG=$(curl -s --max-time 25 "https://$API_HOST/blogs?page=1&per_page=5" 2>/dev/null \
      | grep -oE "https://$MEDIA_HOST/[A-Za-z0-9._-]+" | head -1)
if [ -z "$IMG" ]; then
    echo "  (找不到取樣圖片，略過)"
else
    ORIG=$(curl -s -o /dev/null -w '%{size_download}' --max-time 30 "$IMG")
    echo "  原圖 $ORIG bytes"
    ENC=$(printf '%s' "$IMG" | sed 's|:|%3A|g; s|/|%2F|g')
    PREV=""; SAME=1
    for W in 64 256 1200; do
        S=$(curl -s -o /dev/null -w '%{size_download}' --max-time 40 \
            -H 'Accept: image/avif,image/webp,image/*' \
            "https://$FRONT_HOST/_next/image?url=$ENC&w=$W&q=75")
        printf "    w=%-5s %s bytes\n" "$W" "$S"
        [ -n "$PREV" ] && [ "$S" != "$PREV" ] && SAME=0
        PREV="$S"
    done
    if [ "$SAME" = "1" ]; then
        echo "    ⚠ 各寬度 size 相同 —— sharp 很可能載入失敗，Next 正在靜默回傳原圖"
        echo "      檢查：standalone 產出裡有沒有 @img/sharp-libvips-*/lib/libvips-cpp.so.*"
    else
        echo "    ✓ size 隨寬度變化，最佳化正常"
    fi
fi
