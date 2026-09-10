#!/usr/bin/env bash
# backup.sh — 数据库备份(ADR-A-002 兑现,arch-v2 §7 / 基线 §7.3)。
#
# 产出:pg_dump 自定义格式转储 + 同名 .sha256 校验文件 + 同名 .meta 说明,落在 backups/(已 gitignore)。
# 保留策略:最近 7 个日历日各留最新一份(日备)+ 最近 5 个 ISO 周各留最新一份(周备),
# 其余自动清理。窗口 ≥ 4 周,满足基线的「至少能回退到 7 天前与 1 个月前」——
# 周备取 5 而非 4:4 个 ISO 周最老只保证 22 天,够不到「1 个月前」。
#
# 用法:scripts/backup.sh [--keep-all]
#   --keep-all  只做备份,不清理旧文件(首次上线或手工归档时用)
#
# 为什么是自定义格式而不是纯 SQL:自定义格式**恢复时可选择、可并行、可校验**
# (pg_restore --list 能先验证档案可读),纯 SQL 只有「整份灌回去」一条路。
#
# 为什么写临时文件再改名:半截的转储如果顶着正式文件名留在目录里,
# 下次恢复演练会拿它当有效备份 —— 失败必须留下失败的痕迹,不能留下似是而非的成功。
set -euo pipefail
cd "$(dirname "$0")/.."

KEEP_ALL=0
[ "${1:-}" = "--keep-all" ] && KEEP_ALL=1

BACKUP_DIR="${BACKUP_DIR:-backups}"
DAILY_KEEP=7
WEEKLY_KEEP=5

# 备份名的身份是**文件名里的时间戳**,不是 mtime:拷贝、移动、同步都会改 mtime,
# 而文件名跟着内容走。全脚本只此一处解析,避免两处各写一份解析迟早不一致。
stamp_of() { local n="${1##*/wise_wealth_}"; printf '%s' "${n%.dump}"; }

# ---------- 1. 依赖与配置 ----------
# 只检查备份链路自己**必然要调用**的工具;psql 只用来读 .meta 里的迁移数,
# 缺了它备份照样成立(该字段退化为 unknown),故不在这里拦(基线 §7.5.2 对「探测本机中间件」有禁令)
for tool in pg_dump pg_restore sha256sum; do
  command -v "$tool" >/dev/null 2>&1 || { echo "✗ 缺少 $tool,无法备份。"; exit 1; }
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
  DB_URL="${DATABASE_URL_PROD:?DATABASE_URL_PROD 未配置}"
else
  DB_URL="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置}"
fi

mkdir -p "$BACKUP_DIR"
STAMP="$(date +%Y%m%d_%H%M)"
NAME="wise_wealth_${STAMP}"
DUMP="$BACKUP_DIR/${NAME}.dump"
META="$BACKUP_DIR/${NAME}.meta"
TMP="$BACKUP_DIR/.${NAME}.dump.part"

# 上一次失败留下的半截文件:留一天够排查了,再久只会堆积(它们不匹配保留策略的命名模式)
find "$BACKUP_DIR" -maxdepth 1 -name '.*.dump.part' -mtime +0 -delete 2>/dev/null || true

# ---------- 2. 转储(临时文件 → 校验 → 改名) ----------
# 连接串不打印(含口令,基线 §8.3 日志脱敏)
echo "→ 备份 $( [ "${APP_ENV:-dev}" = prod ] && echo '正式库' || echo 'dev 库' ) …"
if ! pg_dump --format=custom --no-owner --no-privileges --file="$TMP" "$DB_URL"; then
  echo "✗ pg_dump 失败,未产出备份(半截文件留在 ${TMP},一天内自动清理,便于排查)。"
  exit 1
fi
# 档案可读性自检:转储能列目录,才说明它不是半截文件(基线 §7.3「禁止只导出不验证」)
if ! pg_restore --list "$TMP" >/dev/null 2>&1; then
  echo "✗ 转储档案无法读取,已丢弃。"
  rm -f "$TMP"
  exit 1
fi
mv "$TMP" "$DUMP"
( cd "$BACKUP_DIR" && sha256sum "$(basename "$DUMP")" > "${NAME}.dump.sha256" )

# 基线 §7.3 备份内容 = 数据库 + 配置(不含密钥)+ 代码版本:密钥不进备份,
# 配置从 .env 取(同名留一份说明),代码版本记 commit —— 恢复时才知道这份数据配哪版代码。
# 连接串里没有库名段时 sed 会原样返回,那样 DB_NAME 就是**整条连接串(含口令)**——
# 它会跟着进 .meta 并被 echo 出来。取不到就写 unknown,绝不回落到原文。
DB_NAME="$(printf '%s' "$DB_URL" | sed -E 's#.*/([^/?]+)(\?.*)?$#\1#')"
[ "$DB_NAME" = "$DB_URL" ] && DB_NAME="unknown(连接串无可解析的库名)"
case "$DB_NAME" in *:*|*@*) DB_NAME="unknown(连接串无可解析的库名)" ;; esac
COMMIT="$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
git diff --quiet 2>/dev/null || COMMIT="${COMMIT}(有未提交改动)"
MIGRATIONS="unknown"
if command -v psql >/dev/null 2>&1; then
  MIGRATIONS="$(psql "$DB_URL" -tAc 'select coalesce(max(version),0) from _sqlx_migrations' 2>/dev/null || echo unknown)"
fi
{
  echo "时间=$(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "库=${DB_NAME}"
  echo "git=${COMMIT}"
  echo "已应用迁移数=${MIGRATIONS}"
  echo "pg_dump=$(pg_dump --version | awk '{print $3}')"
  # 配置(不含密钥):非密钥项照记,密钥项只记「已配置」—— 基线 §7.3 的备份内容含配置,
  # 而本项目配置就在 .env 里,整份拷进备份等于把口令写进备份文件
  echo "配置.APP_ENV=${APP_ENV:-未设置}"
  echo "配置.APP_PORT=${APP_PORT:-未设置}"
  echo "配置.API_BASE_URL=${API_BASE_URL:-未设置}"
  echo "配置.SESSION_TTL_DAYS=${SESSION_TTL_DAYS:-未设置(缺省 30)}"
  for k in JWT_SECRET SEED_USERNAME SEED_PASSWORD; do
    [ -n "${!k:-}" ] && echo "配置.${k}=已配置(值不入备份)"
  done
  echo "说明=本文件与同名 .dump/.sha256 同一批;密钥类配置改由 .env 保管,不入备份"
} > "$META"

SIZE="$(du -h "$DUMP" | cut -f1)"
echo "✓ 备份完成:${DUMP}(${SIZE})"
echo "  校验文件:${BACKUP_DIR}/${NAME}.dump.sha256"
echo "  版本说明:${META}(git ${COMMIT}、迁移 ${MIGRATIONS})"

# ---------- 3. 保留策略:最近 7 天每天一份 + 最近 4 周每周一份 ----------
# 判据取文件名里的时间戳(见 stamp_of)。「7 天」是 7 个**日历日**各留最新一份,
# 不是「最新的 7 个文件」—— 一天跑五次时后者只覆盖一天多,回退不到 7 天前。
retain() {
  mapfile -t files < <(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump 2>/dev/null | sort)
  local total="${#files[@]}"
  [ "$total" -eq 0 ] && return 0

  # 从最新往回扫:每个日历日、每个 ISO 周各自的第一份就是该日/该周最新的一份
  declare -A keep=() seen_day=() seen_week=()
  local kept_days=0 kept_weeks=0 i f stamp day week
  for (( i = total - 1; i >= 0; i-- )); do
    f="${files[$i]}"
    stamp="$(stamp_of "$f")"
    day="$(date -d "${stamp:0:4}-${stamp:4:2}-${stamp:6:2}" +%F 2>/dev/null || true)"
    week="$(date -d "${stamp:0:4}-${stamp:4:2}-${stamp:6:2}" +%G-%V 2>/dev/null || true)"
    [ -z "$day" ] && continue
    if [ -z "${seen_day[$day]:-}" ] && [ "$kept_days" -lt "$DAILY_KEEP" ]; then
      seen_day[$day]=1; kept_days=$(( kept_days + 1 )); keep["$f"]=1
    fi
    if [ -z "${seen_week[$week]:-}" ] && [ "$kept_weeks" -lt "$WEEKLY_KEEP" ]; then
      seen_week[$week]=1; kept_weeks=$(( kept_weeks + 1 )); keep["$f"]=1
    fi
  done

  # 不在保留集合里的删除(连同校验文件与说明);只动自家命名模式下的文件
  local removed=0
  for f in "${files[@]}"; do
    if [ -z "${keep[$f]:-}" ]; then
      rm -f "$f" "${f%.dump}.dump.sha256" "${f%.dump}.meta"
      echo "  - 清理:${f}"
      removed=$(( removed + 1 ))
    fi
  done
  local left
  left="$(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump 2>/dev/null | wc -l)"
  echo "✓ 保留策略:日备保留 ${kept_days} 个日历日 + 周备保留 ${kept_weeks} 个 ISO 周(现有 ${left} 份,本次清理 ${removed} 份)"
}

if [ "$KEEP_ALL" = 1 ]; then
  echo "· 跳过保留策略清理(--keep-all)"
else
  retain
fi

# ---------- 4. 异地提醒(arch-v2 §7:本地 + 异地各一份) ----------
echo "· 异地副本请手工拷贝到云盘/另一台机器 —— 本地盘上的备份挡不住本地盘的故障。"
echo "· 恢复演练:scripts/restore-drill.sh"
