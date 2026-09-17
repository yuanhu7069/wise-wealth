#!/usr/bin/env bash
# check.sh — 全量质量门(AC-6 等):clippy + cargo test + biome + tsc + Token 门禁 + designmd validate。
# 用法:scripts/check.sh
set -euo pipefail
cd "$(dirname "$0")/.."

fail=0

echo "== 1/5 cargo clippy(全 target,-D warnings)=="
if ! (cd server && SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings); then
  fail=1
fi

echo "== 2/5 cargo test(SQLX_OFFLINE)=="
if ! (cd server && SQLX_OFFLINE=true cargo test); then
  fail=1
fi

echo "== 3/5 biome check(src + scripts)=="
if ! (cd web && npx biome check src scripts); then
  fail=1
fi

echo "== 4/5 tsc --noEmit(类型检查)=="
if ! (cd web && npx tsc --noEmit); then
  fail=1
fi

echo "== 5/6 Token 门禁(RULE-006:禁裸色值/px,globals.css 除外)=="
if ! (cd web && bash scripts/check-tokens.sh); then
  fail=1
fi

echo "== 6/6 designmd validate(DESIGN.md 改造件防漂移,参考设计基线 §3)=="
if ! npx -y designmd.sh@latest validate ./DESIGN.md >/dev/null; then
  fail=1
fi

if [ $fail = 0 ]; then
  echo "✓ check.sh 全绿"
else
  echo "✗ check.sh 存在失败项,见上方日志"
  exit 1
fi
