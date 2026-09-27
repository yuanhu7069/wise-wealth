#!/usr/bin/env bash
# set-tier.sh — 置账号分层 free/plus(H 期 ADR-H-001:产品内无购买路径,唯一开通方式)。
#
# 分层实时生效(RULE-046):置 plus 后无需重新生成方案,下一次请求即得推理链。
# 用法:scripts/set-tier.sh <username> <free|plus>
set -euo pipefail
cd "$(dirname "$0")/.."

if [ $# -ne 2 ]; then
  echo "用法:scripts/set-tier.sh <username> <free|plus>" >&2
  exit 1
fi

username="$1"
tier="$2"
case "$tier" in
  free|plus) ;;
  *) echo "tier 只能是 free 或 plus,收到:$tier" >&2; exit 1 ;;
esac

if [ ! -f .env ]; then
  echo "缺少 .env:请先 cp .env.example .env 并填写。" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1091
source .env
set +a

db_url="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置}"

# 值经 psql 变量绑定(:'var',stdin 路径才做替换;-c 不做 —— 踩过语法错误)传入,不拼 SQL 字面量
psql "$db_url" -v ON_ERROR_STOP=1 -v u="$username" -v t="$tier" <<'SQL'
UPDATE users SET tier = :'t'::text WHERE username = :'u';
SQL

# UPDATE 0 行也退 0(psql 通病):存在性显式确认,账号写错不能静默「成功」
exists=$(printf "SELECT count(*) FROM users WHERE username = :'u';\n" \
  | psql "$db_url" -t -A -v u="$username")
if [ "$exists" = "0" ]; then
  echo "✗ 账号不存在:$username" >&2
  exit 1
fi

echo "✓ $username 的分层已置为 $tier(实时生效,刷新页面即可)"
