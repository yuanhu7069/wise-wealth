#!/usr/bin/env bash
# ticket 08 端到端实测:备份脚本与恢复演练。证据脚本,不是交付物。
#
# 覆盖 ticket 08 的六个验收项:
#   1. 备份脚本产出自定义格式转储 + SHA256 校验文件,并按保留策略清理旧备份
#   2. 备份文件不落在会被提交到版本库的位置
#   3. 恢复步骤可把备份导入副本库并成功启动服务
#   4. 抽验:副本库中档案与方案的行数、关键金额与源库一致
#   5. 演练记录(时间、备份文件、校验值、抽验结果)写入 B 期测试报告
#   6. README／运维说明写明「恢复演练完成前不录入不可重建的真实数据」
#
# 会往 backups/ 写真实备份与合成备份。合成文件登记在册、退出时(含中断)按册删除,
# 绝不按通配符删 —— 通配符有机会误伤真实备份。不写源库。
# 退出码:0 全过;1 有验收项失败;2 演练因**缺少副本库权限**而阻塞(需管理员建库)。
# 注意:2 只在「其余各项全过」时才算阻塞,有失败项一律记 1 —— 别让阻塞掩盖失败。
set -euo pipefail
cd "$(dirname "$0")/../.."

FAIL=0
BLOCKED=0
SYNTH_LIST="$(mktemp)"
cleanup() {
  if [ -s "$SYNTH_LIST" ]; then xargs -r rm -f < "$SYNTH_LIST"; fi
  rm -f "$SYNTH_LIST"
}
trap cleanup EXIT

say() { printf '\n===== %s =====\n' "$1"; }
expect() { # expect <说明> <实际> <期望>
  if [ "$2" = "$3" ]; then echo "  ✓ $1 = $2"; else echo "  ✗ $1:期望 $3,实际 $2"; FAIL=1; fi
}
expect_ge() { # expect_ge <说明> <实际> <下界>:用于「文档里提到了几处」这类会随编辑变动的计数
  if [ "$2" -ge "$3" ] 2>/dev/null; then echo "  ✓ $1 = $2(≥$3)"; else echo "  ✗ $1:期望 ≥$3,实际 $2"; FAIL=1; fi
}
# 末尾必须有换行:没有换行时循环里每行结果会首尾相连成一行,后面的 sort/uniq 全部失效
day_of() { local n="${1##*/wise_wealth_}"; printf '%s-%s-%s\n' "${n:0:4}" "${n:4:2}" "${n:6:2}"; }
# 计数用纯循环,不走管道:管道里「最后一条命令非零」会被 set -o pipefail 判成整条失败
count_day() { local n=0 f; for f in "${BACKUP_DIR:-backups}"/wise_wealth_*.dump; do [ "$(day_of "$f")" = "$1" ] && n=$(( n + 1 )); done; printf '%s' "$n"; }
list_days() { local f; for f in "${BACKUP_DIR:-backups}"/wise_wealth_*.dump; do day_of "$f"; done | sort -r | uniq; }
stamp_of() { local n="${1##*/wise_wealth_}"; printf '%s' "${n%.dump}"; }
mk_synth() { # mk_synth <文件名时间戳>:造一份合成备份(三件套),登记待清理
  local t="$1" f
  for ext in dump dump.sha256 meta; do
    f="${BACKUP_DIR:-backups}/wise_wealth_${t}.${ext}"
    printf 'x' > "$f"
    printf '%s\n' "$f" >> "$SYNTH_LIST"
  done
}

say "1. 备份产出:自定义格式 + SHA256 + 版本说明 + 档案可读"
scripts/backup.sh | sed 's/^/  /'
DUMP="$(ls -1 backups/wise_wealth_*.dump | sort | tail -1)"
expect "转储文件存在" "$([ -f "$DUMP" ] && echo yes)" "yes"
expect "校验文件存在" "$([ -f "${DUMP%.dump}.dump.sha256" ] && echo yes)" "yes"
expect "版本说明存在(基线 §7.3:含代码版本)" "$([ -f "${DUMP%.dump}.meta" ] && echo yes)" "yes"
expect "说明里记了 git 版本" "$(grep -c '^git=' "${DUMP%.dump}.meta")" "1"
expect "说明里记了已应用迁移数" "$(grep -c '^已应用迁移数=' "${DUMP%.dump}.meta")" "1"
expect "说明里记了脱敏配置(基线 §7.3:配置不含密钥)" "$(grep -c '^配置\.APP_ENV=' "${DUMP%.dump}.meta")" "1"
expect "密钥值不入备份" "$(grep -cE '^配置\.(JWT_SECRET|SEED_PASSWORD)=\|(sk-|postgres://)' "${DUMP%.dump}.meta" || true)" "0"
echo "  · .meta 内容:"; sed 's/^/    /' "${DUMP%.dump}.meta"
expect "SHA256 校验通过" "$( ( cd backups && sha256sum -c "$(basename "${DUMP%.dump}").dump.sha256" ) >/dev/null 2>&1; echo $? )" "0"
expect "格式为 CUSTOM" "$(pg_restore --list "$DUMP" 2>/dev/null | grep -c 'Format: CUSTOM' || true)" "1"
expect "档案含 5 张业务表" "$(pg_restore --list "$DUMP" | grep -cE 'TABLE public (users|profiles|plans|plan_buckets|analytics_events)')" "5"
echo "  · 篡改检测:改一个字节后校验必须失败"
cp "$DUMP" "$DUMP.orig" && printf 'x' | dd of="$DUMP" bs=1 seek=200 conv=notrunc status=none
expect "被篡改的备份校验失败" "$(cd backups && sha256sum -c "$(basename "${DUMP%.dump}").dump.sha256" >/dev/null 2>&1; echo $?)" "1"
mv "$DUMP.orig" "$DUMP"

say "2. 保留策略:最近 7 个日历日各一份 + 最近 5 个 ISO 周各一份"
# 在一个**临时目录**里造历史。合成文件绝不落进真实 backups/:它们顶着今天的日期,
# 会被「取最新一份」选中,还可能把当天的真实备份挤掉(两轮评审都点了这里)。
tmpdir="$(mktemp -d)"
export BACKUP_DIR="$tmpdir"
i=1
while [ "$i" -le 20 ]; do mk_synth "$(date -d "-${i} day" +%Y%m%d)_0300"; i=$(( i + 1 )); done
mk_synth "$(date +%Y%m%d)_0100"
mk_synth "$(date +%Y%m%d)_0200"
mk_synth "$(date -d '-35 day' +%Y%m%d)_0300"   # 供下面「最老一份」用:应被周备留住
mk_synth "$(date -d '-60 day' +%Y%m%d)_0300"   # 远在 5 周之外:必须被清掉
TODAY="$(date +%Y%m%d)"
LATEST_TODAY="$(ls -1 "$BACKUP_DIR"/wise_wealth_${TODAY}_*.dump | sort | tail -1)"
scripts/backup.sh >/dev/null

# 属性断言(不复刻脚本的算法,只验规则本身):
# ① 最近 7 个出现过的日历日,每天恰好留 1 份
mapfile -t all_days < <(list_days)
recent_days=("${all_days[@]:0:7}")
expect "出现过的日数(应 ≥8:20 天历史 + 今天)" "$([ "${#all_days[@]}" -ge 8 ] && echo yes)" "yes"
expect "最近保留的日数" "${#recent_days[@]}" "7"
for d in "${recent_days[@]}"; do expect "「$d」保留份数" "$(count_day "$d")" "1"; done
# ② 一天跑多次只留最新那份(今天造了 3 份:0100 / 0200 / 脚本刚跑的)
expect "今天只留 1 份" "$(count_day "$(date +%F)")" "1"
expect "留的正是今天最新那份 $(basename "$LATEST_TODAY")" "$([ -f "$LATEST_TODAY" ] && echo yes)" "yes"
# ③ 远在保留窗口之外的日子一份不留(不敢按「第 N 周」写死:周备按时**有文件存在**的周数,
#    窗口边界随实际文件分布浮动,写死某一周会得出时对时错的结论)
expect "「$(date -d '-60 day' +%F)」(窗口外)保留份数" "$(count_day "$(date -d '-60 day' +%F)")" "0"
# ④ 回退窗口:最老一份的年龄必须够到基线的「1 个月前」(周备 4 份时只有 22 天,不够)
oldest="$(ls -1 "$BACKUP_DIR"/wise_wealth_*.dump | sort | head -1)"
oldest_day="$(day_of "$oldest")"
span="$(( ( $(date -d "$TODAY" +%s) - $(date -d "$oldest_day" +%s) ) / 86400 ))"
expect "最老一份距今(天)" "$([ "$span" -ge 28 ] && echo "≥28(实为 ${span})" || echo "$span")" "≥28(实为 ${span})"
# ⑤ 校验/说明文件不残留孤儿:三件套数量一致
expect "dump 与 sha256 数量一致" "$(ls -1 "$BACKUP_DIR"/*.dump | wc -l)" "$(ls -1 "$BACKUP_DIR"/*.dump.sha256 | wc -l)"
expect "dump 与 meta 数量一致" "$(ls -1 "$BACKUP_DIR"/*.dump | wc -l)" "$(ls -1 "$BACKUP_DIR"/*.meta | wc -l)"
expect "总数不超过 7+5" "$([ "$(ls -1 "$BACKUP_DIR"/*.dump | wc -l)" -le 12 ] && echo yes)" "yes"
unset BACKUP_DIR
rm -rf "$tmpdir"
echo "  · 保留策略在临时目录里验完,真实 backups/ 未被污染(当前 $(ls -1 backups/wise_wealth_*.dump | wc -l) 份)"

say "3. 备份不进版本库"
expect "backups/ 被 gitignore" "$(git check-ignore -q backups/probe.dump && echo yes || echo no)" "yes"
expect "git status 里没有备份文件" "$(git status --porcelain | grep -c '^?? backups/' || true)" "0"

say "4. 恢复演练(备份 → 副本库 → 抽验 → 起服务)"
set +e
scripts/restore-drill.sh > /tmp/ww-drill.log 2>&1
DRILL_RC=$?
set -e
tail -25 /tmp/ww-drill.log | sed 's/^/  /'
if [ "$DRILL_RC" = 3 ]; then
  echo "  ⚠ 阻塞:当前数据库角色无建库权限,副本库需管理员创建(演练本身未跑)"
  BLOCKED=1
else
  expect "演练退出码" "$DRILL_RC" "0"
  expect "恢复完成" "$(grep -c '✓ 恢复完成' /tmp/ww-drill.log || true)" "1"
  expect "副本库服务健康" "$(grep -c '"db":"ok"' /tmp/ww-drill.log || true)" "1"
  expect "抽验项数(行数/金额/指纹共 10)" "$(grep -c '✓$' /tmp/ww-drill.log || true)" "10"
  expect "抽验无 ✗" "$(grep -c '✗' /tmp/ww-drill.log || true)" "0"
  expect "副本库上登录成功" "$(grep -c '✓ 副本库上可用种子账号登录' /tmp/ww-drill.log || true)" "1"
  expect "从副本库读出方案数据" "$(grep -c '副本库上可读出方案数据' /tmp/ww-drill.log || true)" "1"
fi

say "5. 演练记录字段齐备(时间/备份文件/校验值/抽验结果)"
if [ "$BLOCKED" = 0 ]; then
  expect "记录含时间" "$(grep -c '^  时间' /tmp/ww-drill.log || true)" "1"
  expect "记录含备份文件" "$(grep -c '^  备份文件' /tmp/ww-drill.log || true)" "1"
  expect "记录含 SHA256" "$(grep -c '^  SHA256' /tmp/ww-drill.log || true)" "1"
  expect "记录含抽验结果" "$(grep -c '^  抽验结果' /tmp/ww-drill.log || true)" "1"
else
  echo "  · 演练未跑,记录字段留待演练通过后核对"
fi

say "6. README 写明红线与恢复步骤"
expect "README 有「演练完成前不录入」红线" "$(grep -c '恢复演练未通过前,不录入不可重建的真实数据' README.md)" "1"
expect_ge "README 提到备份命令" "$(grep -c 'scripts/backup.sh' README.md || true)" "2"
expect_ge "README 提到恢复演练命令" "$(grep -c 'scripts/restore-drill.sh' README.md || true)" "2"
expect "README 写明升级路径" "$(grep -c '^## 升级' README.md)" "1"
expect "README 写明演练通过后红线何时再生效" "$(grep -c '换库、改迁移脚本或改恢复链路后,红线重新生效' README.md)" "1"

say "结果"
if [ "$FAIL" != 0 ]; then
  echo "✗ 存在未通过项,见上方 ✗(阻塞状态不掩盖失败)"
  exit 1
elif [ "$BLOCKED" = 1 ]; then
  echo "⚠ 备份侧全部通过;恢复演练因缺少副本库权限被阻塞(需管理员建库后重跑本脚本)"
  exit 2
else
  echo "✓ ticket 08 验收项全部通过"
fi
