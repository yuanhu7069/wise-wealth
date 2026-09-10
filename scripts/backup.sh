#!/usr/bin/env bash
# backup.sh — 数据库备份(ADR-A-002 兑现,arch-v2 §7)。
#
# 产出:pg_dump 自定义格式转储 + 同名 .sha256 校验文件,落在 backups/(已 gitignore)。
# 保留策略:日备 7 份 + 周备 4 份,超出部分自动清理。
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
WEEKLY_KEEP=4

# ---------- 1. 依赖与配置 ----------
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
TMP="$BACKUP_DIR/.${NAME}.dump.part"

# ---------- 2. 转储(临时文件 → 校验 → 改名) ----------
# 连接串不打印(含口令,基线 §8.3 日志脱敏)
echo "→ 备份 $( [ "${APP_ENV:-dev}" = prod ] && echo '正式库' || echo 'dev 库' ) …"
if ! pg_dump --format=custom --no-owner --no-privileges --file="$TMP" "$DB_URL"; then
  echo "✗ pg_dump 失败,未产出备份(临时文件保留在 ${TMP},便于排查)。"
  exit 1
fi
# 档案可读性自检:转储能列目录,才说明它不是半截文件
if ! pg_restore --list "$TMP" >/dev/null 2>&1; then
  echo "✗ 转储档案无法读取,已丢弃。"
  rm -f "$TMP"
  exit 1
fi
mv "$TMP" "$DUMP"
( cd "$BACKUP_DIR" && sha256sum "$(basename "$DUMP")" > "${NAME}.dump.sha256" )

SIZE="$(du -h "$DUMP" | cut -f1)"
echo "✓ 备份完成:${DUMP}(${SIZE})"
echo "  校验文件:${BACKUP_DIR}/${NAME}.dump.sha256"

# ---------- 3. 保留策略:日备 7 份 + 周备 4 份 ----------
# 判据取**文件名里的时间戳**,不取 mtime —— 拷贝、移动、同步都会改 mtime,
# 而文件名是备份内容的副本身份。
retain() {
  local files=() f keep=() week last_week=""
  mapfile -t files < <(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump 2>/dev/null | sort)
  local total="${#files[@]}"
  [ "$total" -eq 0 ] && return 0

  # ① 日备:最新的 DAILY_KEEP 份
  local daily_start=$(( total > DAILY_KEEP ? total - DAILY_KEEP : 0 ))
  for (( i = daily_start; i < total; i++ )); do keep+=("${files[$i]}"); done

  # ② 周备:从最新往回数,最近 WEEKLY_KEEP 个 ISO 周各留该周最新的一份
  local weeks=0
  for (( i = total - 1; i >= 0; i-- )); do
    f="${files[$i]}"
    local stamp="${f##*/wise_wealth_}"; stamp="${stamp%.dump}"
    local d="${stamp:0:4}-${stamp:4:2}-${stamp:6:2} ${stamp:9:2}:${stamp:11:2}"
    local week; week="$(date -d "$d" +%G-%V 2>/dev/null || true)"
    [ -z "$week" ] && continue
    if [ "$week" != "$last_week" ]; then
      [ "$weeks" -ge "$WEEKLY_KEEP" ] && break
      weeks=$(( weeks + 1 ))
      last_week="$week"
      # 该周还没被日备收录才加进来(去重)
      local dup=0 k
      for k in "${keep[@]:-}"; do [ "$k" = "$f" ] && dup=1; done
      [ "$dup" -eq 0 ] && keep+=("$f")
    fi
  done

  # ③ 不在保留集合里的删除(连同校验文件);只动自家命名模式下的文件
  local removed=0
  for f in "${files[@]}"; do
    local in_keep=0 k
    for k in "${keep[@]:-}"; do [ "$k" = "$f" ] && in_keep=1; done
    if [ "$in_keep" -eq 0 ]; then
      rm -f "$f" "${f%.dump}.dump.sha256"
      echo "  - 清理:${f}"
      removed=$(( removed + 1 ))
    fi
  done
  echo "✓ 保留策略:日备 ${DAILY_KEEP} 份 + 周备 ${WEEKLY_KEEP} 份(现有 $(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump 2>/dev/null | wc -l) 份,本次清理 ${removed} 份)"
}

if [ "$KEEP_ALL" = 1 ]; then
  echo "· 跳过保留策略清理(--keep-all)"
else
  retain
fi

# ---------- 4. 异地提醒(arch-v2 §7:本地 + 异地各一份) ----------
echo "· 异地副本请手工拷贝到云盘/另一台机器 —— 本地盘上的备份挡不住本地盘的故障。"
echo "· 恢复演练:scripts/restore-drill.sh"
