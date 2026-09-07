#!/usr/bin/env bash
# T4:从后端 /api/v1/openapi.json 生成 src/lib/api-types.ts(RULE-005:生成物禁止手改)。
# 用法:scripts/gen-types.sh [后端地址,默认 http://127.0.0.1:8080]
set -euo pipefail
cd "$(dirname "$0")/.."

BASE_URL="${1:-http://127.0.0.1:8080}"

curl -fsS --max-time 5 "$BASE_URL/api/v1/openapi.json" -o /tmp/wise-wealth-openapi.json
npx openapi-typescript /tmp/wise-wealth-openapi.json \
  -o src/lib/api-types.ts --output-types false
echo "✓ 已生成 src/lib/api-types.ts"
