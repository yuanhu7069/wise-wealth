#!/usr/bin/env bash
# seed-user.sh — 幂等写入/重置种子账号(ADR-B-002:无注册页,忘记口令即重跑本脚本)。
#
# 凭证只经环境变量传入后端子命令,**不进命令行参数**(命令行参数会出现在 ps 输出里)。
# 用法:scripts/seed-user.sh
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f .env ]; then
  echo "缺少 .env:请先 cp .env.example .env 并填写(含 SEED_USERNAME / SEED_PASSWORD)。" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1091
source .env
set +a

if [ -z "${SEED_USERNAME:-}" ] || [ -z "${SEED_PASSWORD:-}" ]; then
  echo "缺少 SEED_USERNAME / SEED_PASSWORD:请在 .env 中配置后重试。" >&2
  exit 1
fi

# 后端只有在 APP_ENV=dev 时要求 DATABASE_URL_DEV;统一映射给 sqlx/sqlx-cli 之外的路径
export APP_ENV="${APP_ENV:-dev}"
export APP_PORT="${APP_PORT:-8080}"
export DATABASE_URL="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置}"

cd server
cargo run --quiet -- seed-user
