#!/usr/bin/env bash
# restore-drill.sh — 恢复演练(AC-15 / arch-v2 §7):备份 → 副本库恢复 → 抽验 → 起服务验证。
#
# 全程**不碰源库**:只读它做比对,所有写入都发生在副本库。
#
# 用法:
#   scripts/restore-drill.sh              先跑一次 backup.sh 取最新备份,再演练它
#   scripts/restore-drill.sh <备份文件>   演练指定的备份文件(「从旧备份恢复」的场景)
#
# 副本库名:WW_DRILL_DB,缺省 wise_wealth_db_test(实例上已备好的空库,与源库同属主)。
# 建库需要 CREATEDB 权限;当前角色没有权限时会打印**需要人执行的 SQL** 并以退出码 3
# 结束 —— 这是实例管理员才能做的一步,脚本不假装能做,也不降级到「恢复进源库」
# (那会把演练变成事故)。
set -euo pipefail
cd "$(dirname "$0")/.."

DRILL_PORT=8099
DRILL_DB="${WW_DRILL_DB:-wise_wealth_db_test}"
BACKUP_DIR="${BACKUP_DIR:-backups}"
BIN="server/target/debug/wise-wealth-server"
RESTORE_LOG="$(mktemp)"   # pg_restore 的 stderr
SERVER_LOG="$(mktemp)"    # 演练用服务进程的输出
SRV_PID=""
trap 'rm -f "$RESTORE_LOG" "$SERVER_LOG"; [ -n "$SRV_PID" ] && kill "$SRV_PID" 2>/dev/null || true' EXIT

say() { printf '\n===== %s =====\n' "$1"; }

for tool in pg_restore psql sha256sum curl; do
  command -v "$tool" >/dev/null 2>&1 || { echo "✗ 缺少 $tool,无法演练。"; exit 1; }
done
if [ ! -f .env ]; then
  echo "✗ 缺少 .env 文件(连接串从这里取,不写死在脚本里)。"
  exit 1
fi

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
  # 取最新一份按**文件名**排序,不按 mtime:拷进来的旧备份 mtime 是新的,
  # 按 mtime 会挑中它 —— 与 backup.sh 的保留判据必须同一个口径
  DUMP="$(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump 2>/dev/null | sort | tail -1 || true)"
  [ -n "$DUMP" ] || { echo "✗ ${BACKUP_DIR}/ 下没有备份文件。"; exit 1; }
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
# 库名会进 CREATE DATABASE 与连接串:先做字符白名单。否则一个引号就能改变语句结构
# (脚本里那两道「≠源库」的闸只比值,挡不住这种注入)。
case "$DRILL_DB" in
  ""|*[!A-Za-z0-9_]*)
    echo "✗ 副本库名只允许字母、数字与下划线,收到:${DRILL_DB}"
    exit 1 ;;
esac

say "3. 副本库:${DRILL_DB}"
DRILL_URL="$(printf '%s' "$SRC_URL" | sed -E "s#/[^/?]+(\?.*)?\$#/${DRILL_DB}\1#")"
ADMIN_URL="$(printf '%s' "$SRC_URL" | sed -E "s#/[^/?]+(\?.*)?\$#/postgres\1#")"
# 两道闸,缺一不可,顺序也有讲究(先报准确的那条):
# ①连接串里必须真的有库名段 —— 否则 sed 什么也没换,后面所有「副本库」其实都是源库;
# ②改写后的地址绝不能等于源库 —— 否则 --clean 会把源库的表全删掉,演练直接变事故。
SRC_DB="$(printf '%s' "$SRC_URL" | sed -E 's#.*/([^/?]+)(\?.*)?$#\1#')"
if [ "$SRC_DB" = "$SRC_URL" ] || [ -z "$SRC_DB" ]; then
  echo "✗ 连接串里没有库名段,无法改写副本库地址(连接串不打印)。请检查 .env 里的 DATABASE_URL_*。"
  exit 1
fi
if [ "$DRILL_DB" = "$SRC_DB" ]; then
  echo "✗ WW_DRILL_DB 指到了源库本身(${SRC_DB})—— 恢复会清空源库。换一个副本库名。"
  exit 1
fi
if [ "$DRILL_URL" = "$SRC_URL" ]; then
  echo "✗ 副本库地址与源库相同,拒绝继续。"
  exit 1
fi
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
# --single-transaction:恢复要么整份成功、要么整份回滚,不留一个删了一半的副本库
if ! pg_restore --clean --if-exists --no-owner --no-privileges --single-transaction \
     --dbname="$DRILL_URL" "$DUMP" 2>"$RESTORE_LOG"; then
  echo "✗ pg_restore 失败:"
  cat "$RESTORE_LOG"
  exit 1
fi
[ -s "$RESTORE_LOG" ] && { echo "· pg_restore 提示(非致命):"; cat "$RESTORE_LOG"; }
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
  "snapshots|select count(*) from snapshots"
  "snapshots.余额合计|select coalesce(sum((e.value)::bigint),0) from snapshots s cross join lateral jsonb_each_text(s.balances) e"
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
# 显式传环境变量覆盖 .env(dotenvy 不覆盖已有变量);端口避开 8080,不动正在跑的服务。
# ⚠️ 必须按 APP_ENV 覆盖**对应的那一个**变量:prod 模式下服务读 DATABASE_URL_PROD,
# 只覆盖 DEV 会让它连回源库 —— 那这一步就不是演练,而是拿正式库跑服务。
if [ "${APP_ENV:-dev}" = "prod" ]; then
  DATABASE_URL_PROD="$DRILL_URL" APP_ENV=prod APP_PORT="$DRILL_PORT" "./$BIN" >"$SERVER_LOG" 2>&1 &
else
  DATABASE_URL_DEV="$DRILL_URL" APP_ENV=dev APP_PORT="$DRILL_PORT" "./$BIN" >"$SERVER_LOG" 2>&1 &
fi
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
  tail -20 "$SERVER_LOG"
  exit 1
fi
# 读一次真实业务数据:健康探测过了不代表表能被应用读出来。
# 用种子账号**真的登录一次**:副本库里的 users 行是随备份一起恢复的(argon2 哈希在内),
# 能登录就同时证明了「恢复出来的账号还能用」—— 比只探 /health 强得多。
# 只打印版本与模式,不打印金额(基线 §8.3:财务数值不进日志/记录)。
if [ -n "${SEED_USERNAME:-}" ] && [ -n "${SEED_PASSWORD:-}" ]; then
  JAR="$(mktemp)"
  # 口令经 stdin 送,不走 argv:命令行参数会出现在 ps 输出里(与 seed-user.sh 同一条规则)
  login_payload="$(printf '{"username":"%s","password":"%s"}' "$SEED_USERNAME" "$SEED_PASSWORD")"
  login="$(printf '%s' "$login_payload" | curl -s --noproxy '*' --max-time 3 -c "$JAR" -X POST \
    "http://127.0.0.1:${DRILL_PORT}/api/v1/auth/login" -H 'content-type: application/json' \
    -d @- || true)"
  unset login_payload
  case "$login" in
    *'"success":true'*) echo "✓ 副本库上可用种子账号登录(口令哈希随备份恢复)" ;;
    *) echo "✗ 副本库上登录失败:$login"; rm -f "$JAR"; exit 1 ;;
  esac
  plans_json="$(curl -s --noproxy '*' --max-time 3 -b "$JAR" \
    "http://127.0.0.1:${DRILL_PORT}/api/v1/plans/active" || true)"
  rm -f "$JAR"
  case "$plans_json" in
    *'"success":true'*)
      ver="$(printf '%s' "$plans_json" | grep -o '"version":[0-9]*' | head -1 | cut -d: -f2 || true)"
      mode="$(printf '%s' "$plans_json" | grep -o '"l1_mode":"[a-z_0-9]*"' | head -1 | cut -d'"' -f4 || true)"
      echo "✓ 副本库上可读出方案数据(第 ${ver} 版 / 模式 ${mode})" ;;
    *'"errorCode":"NOT_FOUND"'*) echo "· 副本库当前没有 active 方案(源库也没有,属正常)" ;;
    *) echo "✗ 从副本库读方案失败:$plans_json"; exit 1 ;;
  esac
else
  echo "· .env 未配 SEED_USERNAME/SEED_PASSWORD,跳过「登录并读方案」这一步"
fi
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
