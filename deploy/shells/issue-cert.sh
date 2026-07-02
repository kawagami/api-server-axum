#!/bin/bash
# 首次取得 Let's Encrypt 憑證（在 deploy 目錄執行，之後 certbot 容器自動 renew）

docker compose run --rm --entrypoint certbot certbot certonly \
  --dns-cloudflare \
  --dns-cloudflare-credentials /etc/cloudflare/cloudflare.ini \
  --email joelai1988@gmail.com \
  --agree-tos \
  --no-eff-email \
  -d kawa.homes \
  -d "*.kawa.homes"
