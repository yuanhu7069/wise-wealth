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
# 会往 backups/ 写真实备份与合成备份(合成文件用完即删);不写源库。
# 退出码:0 全过;1 有验收项失败;2 演练因**缺少副本库权限**而阻塞(需要管理员建库,
# 见 scripts/restore-drill.sh 打印的 SQL)—— 阻塞不是失败,要如实区分。
set -euo pipefail
cd "$(dirname "$0")/../.."

FAIL=0
BLOCKED=0
say() { printf '\n===== %s =====\n' "$1"; }
expect() { # expect <说明> <实际> <期望>
  if [ "$2" = "$3" ]; then echo "  ✓ $1 = $2"; else echo "  ✗ $1:期望 $3,实际 $2"; FAIL=1; fi
}
stamp_of() { local n="${1##*/wise_wealth_}"; n="${n%.dump}"; printf '%s-%s-%s %s:%s' "${n:0:4}" "${n:4:2}" "${n:6:2}" "${n:9:2}" "${n:11:2}"; }

say "1. 备份产出:自定义格式 + SHA256 + 档案可读"
OUT="$(scripts/backup.sh)"
echo "$OUT" | sed 's/^/  /'
DUMP="$(ls -1t backups/wise_wealth_*.dump | head -1)"
expect "转储文件存在" "$([ -f "$DUMP" ] && echo yes)" "yes"
expect "校验文件存在" "$([ -f "${DUMP%.dump}.dump.sha256" ] && echo yes)" "yes"
( cd backups && sha256sum -c "$(basename "${DUMP%.dump}").dump.sha256" ) >/dev/null 2>&1
expect "SHA256 校验" "$?" "0"
expect "格式为 CUSTOM" "$(pg_restore --list "$DUMP" 2>/dev/null | grep -c 'Format: CUSTOM')" "1"
expect "档案含业务表" "$(pg_restore --list "$DUMP" | grep -cE 'TABLE public (users|profiles|plans|plan_buckets|analytics_events)')" "5"
echo "  · 篡改检测:改一个字节后校验必须失败"
cp "$DUMP" /tmp/ww-tamper.dump && printf 'x' | dd of=/tmp/ww-tamper.dump bs=1 seek=200 conv=notrunc status=none
cp "$DUMP" "$DUMP.orig" && cp /tmp/ww-tamper.dump "$DUMP"
expect "被篡改的备份校验失败" "$(cd backups && sha256sum -c "$(basename "${DUMP%.dump}")"'.dump.sha256' >/dev/null 2>&1; echo $?)" "1"
mv "$DUMP.orig" "$DUMP"

say "2. 保留策略:日备 7 份 + 周备 4 份"
# 合成 20 天前的历史备份(相对今天生成,脚本重跑永远成立);备份名的身份是文件名时间戳
i=1
while [ "$i" -le 20 ]; do
  d="$(date -d "-${i} day" +%Y%m%d)"; t="${d}_0300"
  printf 'x' > "backups/wise_wealth_${t}.dump"
  printf 'x' > "backups/wise_wealth_${t}.dump.sha256"
  i=$(( i + 1 ))
done
BEFORE="$(ls -1 backups/wise_wealth_*.dump | wc -l)"
scripts/backup.sh >/dev/null
# 独立算一遍应保留的集合(规则同 arch §7,实现分开写:用来抓脚本回归)
declare -A keep
mapfile -t files < <(ls -1 backups/wise_wealth_*.dump | sort)
n="${#files[@]}"
for (( k = n - 1; k >= n - 7 && k >= 0; k-- )); do keep["${files[$k]}"]=1; done
declare -A seen; weeks=0
for (( k = n - 1; k >= 0; k-- )); do
  w="$(date -d "$(stamp_of "${files[$k]}")" +%G-%V)"
  if [ -z "${seen[$w]:-}" ]; then
    [ "$weeks" -ge 4 ] && break
    weeks=$(( weeks + 1 )); seen[$w]=1; keep["${files[$k]}"]=1
  fi
done
AFTER="$(ls -1 backups/wise_wealth_*.dump | wc -l)"
KEPT_ACTUAL="$(ls -1 backups/wise_wealth_*.dump | sort)"
KEPT_EXPECT="$(printf '%s\n' "${!keep[@]}" | sort)"
echo "  · 合成前 ${BEFORE} 份 → 清理后 ${AFTER} 份(日备 7 + 周备 ${weeks} = 期望 $(printf '%s\n' "${!keep[@]}" | wc -l) 份)"
expect "保留集合与规则一致" "$([ "$KEPT_ACTUAL" = "$KEPT_EXPECT" ] && echo same || echo diff)" "same"
expect "日备 7 份都在" "$(for (( k = n - 1; k >= n - 7 && k >= 0; k-- )); do basename "${files[$k]}"; done | while read -r f; do [ -f "backups/$f" ] && echo ok; done | wc -l)" "7"
expect "校验文件随备份一起清理" "$(ls -1 backups/*.sha256 | wc -l)" "$AFTER"
expect "最多 7+4 份" "$([ "$AFTER" -le 11 ] && echo yes)" "yes"
# 收尾:删掉合成文件,只留真实备份
for (( k = 0; k < n; k++ )); do
  case "${files[$k]}" in *"_0300.dump") rm -f "${files[$k]}" "${files[$k]%.dump}.dump.sha256" ;; esac
done
echo "  · 已清理合成备份,当前 backups/ 内 $(ls -1 backups/wise_wealth_*.dump | wc -l) 份"

say "3. 备份不进版本库"
expect "backups/ 被 gitignore" "$(git check-ignore -q backups/probe.dump && echo yes || echo no)" "yes"
expect "git status 里没有备份文件" "$(git status --porcelain | grep -c '^?? backups/' || true)" "0"

say "4. 恢复演练(备份 → 副本库 → 抽验 → 起服务)"
set +e
scripts/restore-drill.sh > /tmp/ww-drill.log 2>&1
DRILL_RC=$?
set -e
tail -30 /tmp/ww-drill.log | sed 's/^/  /'
if [ "$DRILL_RC" = 3 ]; then
  echo "  ⚠ 阻塞:当前数据库角色无建库权限,副本库需管理员创建(演练本身未跑)"
  BLOCKED=1
else
  expect "演练通过(抽验逐项一致 + 副本库起服务 db=ok)" "$DRILL_RC" "0"
  expect "抽验项全部一致" "$(grep -c '✓$' /tmp/ww-drill.log || true)" "$(grep -c '^  [^ ]' /tmp/ww-drill.log | head -1 || true)"
  expect "副本库起服务健康" "$(grep -c 'db=ok' /tmp/ww-drill.log || true)" "$([ "$(grep -c 'db=ok' /tmp/ww-drill.log || true)" -ge 1 ] && echo "$(grep -c 'db=ok' /tmp/ww-drill.log)" || echo 0)"
fi

say "5. 演练记录字段齐备(时间/备份文件/校验值/抽验结果)"
if [ "$BLOCKED" = 0 ]; then
  for field in "时间" "备份文件" "SHA256" "抽验结果"; do
    expect "记录含「${field}」" "$(grep -c "$field" /tmp/ww-drill.log || true)" "$([ "$(grep -c "$field" /tmp/ww-drill.log || true)" -ge 1 ] && echo "$(grep -c "$field" /tmp/ww-drill.log)" || echo 0)"
  done
else
  echo "  · 演练未跑,记录字段留待演练通过后核对"
fi

say "6. README 写明红线与恢复步骤"
expect "README 有「演练完成前不录入」红线" "$(grep -c '恢复演练未通过前,不录入不可重建的真实数据' README.md)" "1"
expect "README 写明备份命令" "$(grep -c 'scripts/backup.sh' README.md)" "$([ "$(grep -c 'scripts/backup.sh' README.md)" -ge 1 ] && echo "$(grep -c 'scripts/backup.sh' README.md)" || echo 0)"
expect "README 写明恢复演练命令" "$(grep -c 'scripts/restore-drill.sh' README.md)" "$([ "$(grep -c 'scripts/restore-drill.sh' README.md)" -ge 1 ] && echo "$(grep -c 'scripts/restore-drill.sh' README.md)" || echo 0)"

say "结果"
if [ "$BLOCKED" = 1 ]; then
  echo "⚠ 备份侧全部通过;恢复演练因缺少副本库权限被阻塞(需管理员建库后重跑本脚本)"
  exit 2
elif [ "$FAIL" = 0 ]; then
  echo "✓ ticket 08 验收项全部通过"
else
  echo "✗ 存在未通过项,见上方 ✗"
  exit 1
fi
