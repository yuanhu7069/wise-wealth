#!/usr/bin/env bash
# start.sh — 一键启动前后端(AC-1:全新 shell 执行 ≤60s 双服务在线)。
# 流程:依赖版本检查 → .env/RULE-002 检查(交给后端自身校验) → sqlx migrate run → 并行起后端+前端。
# 用法:scripts/start.sh [--no-frontend]
#   后端  http://127.0.0.1:8080 (/health)
#   前端  http://127.0.0.1:3000
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT="$(pwd)"

WITH_FRONTEND=1
[ "${1:-}" = "--no-frontend" ] && WITH_FRONTEND=0

# ---------- 1. 工具版本检查(缺失即逐条列出并退出) ----------
missing=()
command -v cargo >/dev/null 2>&1 || missing+=("cargo(Rust toolchain,server/rust-toolchain.toml 锁定 1.94.0)")
command -v node  >/dev/null 2>&1 || missing+=("node(>=22)")
command -v npm   >/dev/null 2>&1 || missing+=("npm(>=10)")
command -v sqlx  >/dev/null 2>&1 || missing+=("sqlx-cli(数据库迁移)")
if [ ${#missing[@]} -gt 0 ]; then
  echo "✗ 缺少以下工具,无法启动:"
  for m in "${missing[@]}"; do echo "  - $m"; done
  exit 1
fi

# ---------- 2. .env 检查(RULE-002 具体缺失键由后端 config 校验逐条列出) ----------
if [ ! -f .env ]; then
  echo "✗ 缺少 .env 文件(RULE-002:必须显式配置,禁止隐式默认)。"
  echo "  请参考 .env.example 复制:cp .env.example .env,并填入 DATABASE_URL_DEV / DATABASE_URL_PROD。"
  exit 1
fi

# ---------- 3. 构建产物检查(无则现编,首次可能超出 60s 预算) ----------
if [ ! -x server/target/debug/wise-wealth-server ]; then
  echo "→ 后端二进制不存在,先编译(cargo build)…"
  (cd server && cargo build)
fi
if [ "$WITH_FRONTEND" = 1 ] && [ ! -d web/node_modules ]; then
  echo "→ web 依赖未安装,先 npm install…"
  (cd web && npm install --legacy-peer-deps)
fi

# ---------- 4. 数据库迁移(显式 source .env:sqlx CLI 只读当前目录 dotenv) ----------
echo "→ 执行数据库迁移(sqlx migrate run)…"
set -a
# shellcheck disable=SC1091
source .env
set +a
_app_env="${APP_ENV:-dev}"
if [ "$_app_env" = "prod" ]; then
  export DATABASE_URL="${DATABASE_URL_PROD:?DATABASE_URL_PROD 未配置(RULE-002)}"
else
  export DATABASE_URL="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置(RULE-002)}"
fi
(cd server && SQLX_OFFLINE=false sqlx migrate run)

# ---------- 5. 并行启动 ----------
pids=()
echo "→ 启动后端 :8080 …"
(cd server && APP_ENV=dev ./target/debug/wise-wealth-server) &
pids+=($!)

if [ "$WITH_FRONTEND" = 1 ]; then
  echo "→ 启动前端 :3000 …"
  (cd web && npm run dev) &
  pids+=($!)
fi

# ---------- 6. 就绪探测(≤60s 预算;--noproxy 防宿主代理劫持 127.0.0.1) ----------
CURL_LOCAL=(curl -fsS --noproxy '*' --max-time 2)
deadline=$((SECONDS + 60))
backend_ok=0
frontend_ok=0
while [ $SECONDS -lt $deadline ]; do
  if [ $backend_ok = 0 ] && "${CURL_LOCAL[@]}" http://127.0.0.1:8080/api/v1/health >/dev/null 2>&1; then
    backend_ok=1
    echo "✓ 后端在线 http://127.0.0.1:8080/api/v1/health"
  fi
  if [ "$WITH_FRONTEND" = 1 ] && [ $frontend_ok = 0 ] && "${CURL_LOCAL[@]}" http://127.0.0.1:3000 >/dev/null 2>&1; then
    frontend_ok=1
    echo "✓ 前端在线 http://127.0.0.1:3000"
  fi
  [ $backend_ok = 1 ] && { [ "$WITH_FRONTEND" = 0 ] || [ $frontend_ok = 1 ]; } && break
  sleep 1
done

elapsed=$SECONDS
if [ $backend_ok = 1 ] && { [ "$WITH_FRONTEND" = 0 ] || [ $frontend_ok = 1 ]; }; then
  echo "✓ 双服务就绪(耗时 ${elapsed}s ≤ 60s 预算)。Ctrl+C 一起停止。"
  trap 'kill "${pids[@]}" 2>/dev/null || true' EXIT INT TERM
  wait
else
  echo "✗ 60s 内服务未全部就绪(backend=$backend_ok frontend=$frontend_ok)。请查看上方日志。"
  kill "${pids[@]}" 2>/dev/null || true
  exit 1
fi
