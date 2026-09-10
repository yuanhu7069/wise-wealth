#!/usr/bin/env bash
# restore-drill.sh — 恢复演练(AC-15 / arch-v2 §7):备份 → 副本库恢复 → 抽验 → 起服务验证。
#
# 全程**不碰源库**:只读它做比对,所有写入都发生在副本库。
#
# 用法:
#   scripts/restore-drill.sh              先跑一次 backup.sh 取最新备份,再演练它
#   scripts/restore-drill.sh <备份文件>   演练指定的备份文件(「从旧备份恢复」的场景)
#
# 副本库名:WW_DRILL_DB,缺省 wise_wealth_backup_test(arch-v2 §7 点名的库名)。
# 建库需要 CREATEDB 权限;当前角色没有权限时会打印**需要人执行的 SQL** 并以退出码 3
# 结束 —— 这是实例管理员才能做的一步,脚本不假装能做,也不降级到「恢复进源库」
# (那会把演练变成事故)。
set -euo pipefail
cd "$(dirname "$0")/.."

DRILL_PORT=8099
DRILL_DB="${WW_DRILL_DB:-wise_wealth_backup_test}"
BIN="server/target/debug/wise-wealth-server"
TMPLOG="$(mktemp)"
SRV_PID=""
trap 'rm -f "$TMPLOG"; [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true' EXIT

say() { printf '\n===== %s =====\n' "$1"; }

for tool in pg_restore psql sha256sum curl; do
  command -v "$tool" >/dev/null 2>&1 || { echo "✗ 缺少 $tool,无法演练。"; exit 1; }
done

set -a
# shellcheck disable=SC1091
source .env
set +a
if [ "${APP_ENV:-dev}" = "prod" ]; then
  SRC_URL="${DATABASE_URL_PROD:?DATABASE_URL_PROD 未配置}"
else
  SRC_URL="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置}"
fi

# ---------- 1. 取备份文件 ----------
say "1. 取备份文件"
if [ $# -ge 1 ]; then
  DUMP="$1"
  [ -f "$DUMP" ] || { echo "✗ 备份文件不存在:$DUMP"; exit 1; }
  echo "· 演练指定文件(提示:抽验比对的是「此刻的源库」,若该备份生成较早,"
  echo "  期间又写过库,行数与金额不一致是**预期**的 —— 那时看的是能否恢复,不是等不等)"
else
  scripts/backup.sh
  DUMP="$(ls -1t backups/wise_wealth_*.dump 2>/dev/null | head -1 || true)"
  [ -n "$DUMP" ] || { echo "✗ backups/ 下没有备份文件。"; exit 1; }
fi
echo "· 备份文件:${DUMP}"
BACKUP_DIR="$(dirname "$DUMP")"
SHA_FILE="${DUMP%.dump}.dump.sha256"

# ---------- 2. 校验值 ----------
say "2. 校验备份完整性(SHA256)"
[ -f "$SHA_FILE" ] || { echo "✗ 缺少校验文件 ${SHA_FILE} —— 没有校验值的备份不算备份。"; exit 1; }
( cd "$BACKUP_DIR" && sha256sum -c "$(basename "$SHA_FILE")" )
SHA="$(cut -d' ' -f1 < "$SHA_FILE")"

# ---------- 3. 副本库就位 ----------
say "3. 副本库:${DRILL_DB}"
DRILL_URL="$(printf '%s' "$SRC_URL" | sed -E "s#/[^/?]+(\?.*)?\$#/${DRILL_DB}\1#")"
ADMIN_URL="$(printf '%s' "$SRC_URL" | sed -E "s#/[^/?]+(\?.*)?\$#/postgres\1#")"
if psql "$DRILL_URL" -tAc 'select 1' >/dev/null 2>&1; then
  echo "· 已存在,恢复时用 --clean 覆盖其内容"
else
  echo "· 不存在,尝试创建 …"
  if psql "$ADMIN_URL" -v ON_ERROR_STOP=1 -c "CREATE DATABASE \"$DRILL_DB\";" >/dev/null 2>&1; then
    echo "· 已创建"
  else
    ROLE="$(psql "$SRC_URL" -tAc 'select current_user;')"
    cat <<EOF
✗ 当前数据库角色($ROLE)无权建库,演练无法继续。

  需要实例管理员执行以下**任一条**(二选一),然后重跑本脚本:
    A. 建好副本库(一次性)。**必须带 OWNER**:PostgreSQL 15 起 public schema 归库主所有,
       不指定属主的话应用角色在库里建不了表,恢复照样失败。
         CREATE DATABASE "$DRILL_DB" OWNER "$ROLE";
    B. 给应用角色建库权限(之后脚本可自建自用、可反复演练,权限比 A 宽):
         ALTER ROLE "$ROLE" CREATEDB;

  也可以指向另一个你有权覆盖的库:
    WW_DRILL_DB=<库名> scripts/restore-drill.sh   # 该库 public 下的内容会被覆盖
EOF
    exit 3
  fi
fi

# ---------- 4. 恢复 ----------
say "4. 恢复到副本库(pg_restore --clean)"
# --no-owner/--no-privileges:副本库未必有与源库相同的角色,恢复不该因「属主不存在」而失败
if ! pg_restore --clean --if-exists --no-owner --no-privileges \
     --dbname="$DRILL_URL" "$DUMP" 2>"$TMPLOG"; then
  echo "✗ pg_restore 失败:"
  cat "$TMPLOG"
  exit 1
fi
[ -s "$TMPLOG" ] && { echo "· pg_restore 提示(非致命):"; cat "$TMPLOG"; }
echo "✓ 恢复完成"

# ---------- 5. 抽验:副本库与源库逐项一致 ----------
say "5. 抽验(行数 / 金额 / 内容指纹)"
FAIL=0
# 比的是**数据本身**,不是「能不能查」:行数一样而金额错位,是备份最危险的失效形态
QUERIES=(
  "users|select count(*) from users"
  "profiles|select count(*) from profiles"
  "plans|select count(*) from plans"
  "plan_buckets|select count(*) from plan_buckets"
  "analytics_events|select count(*) from analytics_events"
  "profiles.金额合计(收入/固定支出/存款)|select coalesce(sum(inflow_cents),0)||'/'||coalesce(sum(expense_fixed_monthly_cents),0)||'/'||coalesce(sum(savings_cents),0) from profiles"
  "plan_buckets.金额合计(月转入/目标)|select coalesce(sum(amount_monthly_cents),0)||'/'||coalesce(sum(coalesce(target_cents,0)),0) from plan_buckets"
  "plans.版本与可投资额|select coalesce(max(version),0)||'/'||coalesce(sum(investable_monthly_cents),0) from plans"
  "plans.快照指纹|select coalesce(md5(string_agg(id::text||version||l1_mode||profile_snapshot_json::text,'|' order by id)),'-') from plans"
  "profiles.档案指纹|select coalesce(md5(string_agg(id::text||coalesce(horizon,'')||coalesce(inflow_cents::text,'')||draft_step,'|' order by id)),'-') from profiles"
)
printf '  %-40s %-18s %-18s %s\n' "项" "源库" "副本库" "结果"
for q in "${QUERIES[@]}"; do
  label="${q%%|*}"; sql="${q#*|}"
  src="$(psql "$SRC_URL" -tAc "$sql")"
  dst="$(psql "$DRILL_URL" -tAc "$sql")"
  if [ "$src" = "$dst" ]; then
    printf '  %-40s %-18s %-18s ✓\n' "$label" "$src" "$dst"
  else
    printf '  %-40s %-18s %-18s ✗\n' "$label" "$src" "$dst"
    FAIL=1
  fi
done
[ "$FAIL" = 0 ] || { echo "✗ 抽验不一致 —— 恢复出来的数据与源库不同,按事故处理。"; exit 1; }

# ---------- 6. 起服务验证(恢复只是第一步,能跑起来才算数) ----------
say "6. 用副本库起一次服务"
if [ ! -x "$BIN" ]; then
  echo "· 后端二进制不存在,先编译 …"
  (cd server && SQLX_OFFLINE=true cargo build --quiet)
fi
# 显式传环境变量覆盖 .env(dotenvy 不覆盖已有变量);端口避开 8080,不动正在跑的服务
APP_ENV="${APP_ENV:-dev}" APP_PORT="$DRILL_PORT" DATABASE_URL_DEV="$DRILL_URL" \
  "./$BIN" >"$TMPLOG" 2>&1 &
SRV_PID=$!
ok=0
for _ in $(seq 1 20); do
  body="$(curl -s --noproxy '*' --max-time 2 "http://127.0.0.1:${DRILL_PORT}/api/v1/health" || true)"
  case "$body" in *'"db":"ok"'*) ok=1; break ;; esac
  sleep 0.5
done
if [ "$ok" = 1 ]; then
  echo "✓ 副本库上的服务健康探测 db=ok:$body"
else
  echo "✗ 副本库上的服务未就绪:"
  tail -20 "$TMPLOG"
  exit 1
fi
# 读一次真实业务数据:健康探测过了不代表表能被应用读出来
plans_json="$(curl -s --noproxy '*' --max-time 3 "http://127.0.0.1:${DRILL_PORT}/api/v1/plans/active" || true)"
case "$plans_json" in
  *'"success":true'*) echo "✓ 副本库上可读出方案数据($(printf '%s' "$plans_json" | head -c 60)…)" ;;
  *'"errorCode":"NOT_FOUND"'*) echo "· 副本库当前没有 active 方案(源库也没有,属正常)" ;;
  *) echo "✗ 从副本库读方案失败:$plans_json"; exit 1 ;;
esac
kill "$SRV_PID" 2>/dev/null || true
SRV_PID=""
echo "· 服务已停止;副本库 ${DRILL_DB} 保留着,可自行 drop"

# ---------- 7. 演练记录(可直接抄进 docs/b/test-report-b.md) ----------
say "7. 演练记录"
cat <<EOF
  时间        $(date '+%Y-%m-%d %H:%M %Z')
  源库        $([ "${APP_ENV:-dev}" = prod ] && echo '正式库' || echo 'dev 库')
  备份文件    ${DUMP}
  SHA256      ${SHA}
  副本库      ${DRILL_DB}(恢复后 ${DRILL_PORT} 端口起服务,/health db=ok、可读方案数据)
  抽验结果    行数 / 金额合计 / 内容指纹 共 ${#QUERIES[@]} 项,全部一致
EOF
echo
echo "✓ 恢复演练通过(AC-15)"
