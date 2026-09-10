#!/usr/bin/env bash
# ticket 06 端到端实测:版本化重算 + 首页双形态。证据脚本,不是交付物。
#
# 覆盖 ticket 06 的六个验收项中可自动化的部分:
#   1. 重新生成 → 版本 +1,旧版本仍在库中且不再是 active;「单一 active」由**数据库层**索引保证
#   2. 重新生成走问卷时,上一次的答案由服务端档案预填(此处断言档案未被清空)
#   3. 首页空态:图标 + 一句说明 + 「开始问卷」引导(三要素)
#   4. 首页有方案:每月可投资金额 + 应急金状态 + 进入完整方案/重新生成两个入口
#   5. Token 样例区与 A 期服务状态卡片已不在首页正文
#   6. 首页页脚保留健康状态与一行免责声明
#
# 口令从 .env 取,不打印。**本脚本会先清掉种子账号的方案行**——首页空态需要「还没生成过
# 方案」的真实状态,种子账号本就是可重建的验证数据(scripts/seed-user.sh 幂等重置)。
# 脚本结束时该账号停在「第 3 版 + 存款 300,000」(第 10 节的达标分支);重跑会重设步 5,无需手工清理。
#
# 前置:后端 :8080 与前端 :3000 都在跑(scripts/start.sh)。
set -euo pipefail
cd "$(dirname "$0")"
set -a
# shellcheck disable=SC1091
source ../../.env
set +a

BASE="http://127.0.0.1:8080/api/v1"
WEB="http://127.0.0.1:3000"
DB="${DATABASE_URL_DEV:?DATABASE_URL_DEV 未配置}"
JAR=$(mktemp)
TMP=$(mktemp -d)
trap 'rm -rf "$JAR" "$TMP"' EXIT

say() { printf '\n===== %s =====\n' "$1"; }
psqlq() { psql "$DB" -v ON_ERROR_STOP=1 -tAq "$@"; }

say "0. 前置:后端与前端在线"
curl -s --noproxy '*' -o /dev/null -w '后端 /health → HTTP %{http_code}\n' "$BASE/health"
curl -s --noproxy '*' -o /dev/null -w '前端 / → HTTP %{http_code}\n' "$WEB/login"

say "1. 登录(种子账号)"
curl -s --noproxy '*' -c "$JAR" -X POST "$BASE/auth/login" \
  -H 'content-type: application/json' \
  -d "{\"username\":\"$SEED_USERNAME\",\"password\":\"$SEED_PASSWORD\"}" | head -c 120
echo

say "2. 重置该用户的方案数据(便于验证首页空态)"
psqlq -c "DELETE FROM plan_buckets WHERE plan_id IN (
            SELECT p.id FROM plans p JOIN users u ON u.id = p.user_id
            WHERE u.username = '$SEED_USERNAME');"
psqlq -c "DELETE FROM plans WHERE user_id = (
            SELECT id FROM users WHERE username = '$SEED_USERNAME');"
echo "已清空(重置到「还没生成过方案」)"

say "3. 首页空态(EMPTY-001 三要素)"
curl -s --noproxy '*' -b "$JAR" "$WEB/" -o "$TMP/home-empty.html"
python3 - "$TMP/home-empty.html" <<'PY'
import pathlib, sys
html = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
checks = {
    "空态标题「还没有方案」": "还没有方案" in html,
    "一句说明(约 2 分钟)": "约 2 分钟" in html,
    "引导操作「开始问卷」": "开始问卷" in html and 'href="/questionnaire"' in html,
    "图标(内联 SVG,非文字占位)": "<svg" in html,
    "页脚:健康状态": "服务在线" in html,
    "页脚:一行免责声明 + 完整声明折叠": "完整声明" in html,
    "A 期 Token 样例区已移除": "token-samples" not in html and "Token 样例" not in html,
    "A 期服务状态卡片已移除(正文)": "服务在线 · 数据库已连接" not in html,
}
for name, ok in checks.items():
    print(f'  {"✓" if ok else "✗"} {name}')
assert all(checks.values()), "空态断言失败"
print("✓ 首页空态:三要素齐全,A 期验证产物不在首页")
PY

say "4. 金例 A 问卷分步保存(步 1-5)"
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":1,"horizon":"y5_10"}' -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":2,"drawdown_response":"hold"}' -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":3,"income_stability":"volatile"}' -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":4,"has_social_security":true,"has_commercial_insurance":false,"dependents":1}' -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":5,"inflow_cents":1200000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' \
  -o /dev/null
echo "步 1-5 已保存(月收入 12,000 / 固定支出 4,500 / 存款 24,000)"

say "5. 首次生成方案"
curl -s --noproxy '*' -b "$JAR" -X POST "$BASE/plans" -H 'content-type: application/json' \
  -d '{"l1_mode":"four_accounts"}' -o "$TMP/plan-v1.json" -w 'HTTP %{http_code}\n'

say "6. 首页有方案形态(摘要 + 两个入口)"
curl -s --noproxy '*' -b "$JAR" "$WEB/" -o "$TMP/home-plan.html"
python3 - "$TMP/home-plan.html" <<'PY'
import pathlib, sys
html = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
checks = {
    "每月可投资标签": "每月可投资" in html,
    "投资金额(金例 A ¥800.00)": "800.00" in html,
    "模式名(四账户理财法)": "四账户理财法" in html,
    "应急金状态(未达标口径)": "应急金当前约" in html and "还差约" in html,
    "入口:查看完整方案 → P04": 'href="/plan"' in html,
    "入口:重新生成 → P03": "重新生成" in html and 'href="/questionnaire"' in html,
    "空态不再出现": "还没有方案" not in html,
}
for name, ok in checks.items():
    print(f'  {"✓" if ok else "✗"} {name}')
assert all(checks.values()), "有方案形态断言失败"
print("✓ 首页有方案形态:金额、应急金状态与两个入口齐备")
PY

say "7. 改收入后重新生成(版本化)"
# 收入 12,000 → 15,000:重新生成前先按步 5 提交新答案(模拟「重新生成 → 进问卷改数字」)
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":5,"inflow_cents":1500000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' \
  -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X POST "$BASE/plans" -H 'content-type: application/json' \
  -d '{"l1_mode":"four_accounts"}' -o "$TMP/plan-v2.json" -w 'HTTP %{http_code}\n'
curl -s --noproxy '*' -b "$JAR" "$BASE/plans/active" -o "$TMP/plan-active.json" -w '读回 /plans/active → HTTP %{http_code}\n'

psqlq -c "SELECT version, is_active, investable_monthly_cents
          FROM plans WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME')
          ORDER BY version;" | sed 's/^/  plans 行(version|is_active|investable): /'

python3 - "$TMP/plan-v1.json" "$TMP/plan-v2.json" "$TMP/plan-active.json" <<'PY'
import json, sys
v1 = json.load(open(sys.argv[1]))["data"]
v2 = json.load(open(sys.argv[2]))["data"]
act = json.load(open(sys.argv[3]))["data"]

print(f'版本:{v1["version"]} → {v2["version"]};每月可投资:{v1["investable_monthly_cents"]/100:,.2f} → {v2["investable_monthly_cents"]/100:,.2f}')
assert v1["version"] == 1 and v2["version"] == 2, (v1["version"], v2["version"])
assert v1["id"] != v2["id"], "第二次生成必须插新行,不能覆盖"
assert v2["investable_monthly_cents"] != v1["investable_monthly_cents"], "收入变了,方案必须跟着变"
assert act["id"] == v2["id"] and act["version"] == 2, "读回的 active 必须是新版本"
print("✓ 重新生成:版本 +1、新行落库、active 指向新版本")
PY

say "8. 数据库层不变量(不依赖应用层自觉)"
ROWS=$(psqlq -c "SELECT count(*) || '|' || count(*) FILTER (WHERE is_active)
                 FROM plans WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME');")
echo "  该用户方案行 总数|active 数 = $ROWS"
[ "$ROWS" = "2|1" ] || { echo "  ✗ 期望 2|1(两版都在库中,恰好一版 active)"; exit 1; }
echo "  ✓ 旧版本仍在库中且不再是 active"

echo "  → 直接 INSERT 第二条 active(绕过应用层),期望被唯一索引拒绝:"
# 注意 pipefail:psql 注定失败,必须 `|| true` 之后再看它说了什么,否则管道整体非零会让断言恒假
if { psqlq -c "INSERT INTO plans (id, user_id, l1_mode, l2_mode, version, is_active,
                                 investable_monthly_cents, profile_snapshot_json,
                                 l2_allocation_json, emergency_json)
               VALUES (gen_random_uuid(),
                       (SELECT id FROM users WHERE username = '$SEED_USERNAME'),
                       'four_accounts', 'x', 999, TRUE, 0, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb);" 2>&1 || true; } \
   | grep -q "plans_one_active_per_user"; then
  echo "  ✓ 被 plans_one_active_per_user 拒绝(单一 active 由数据库保证)"
else
  echo "  ✗ 竟然插进去了 —— 数据库层约束失效"
  exit 1
fi

echo "  → 同一用户重复版本号,期望被 plans_user_version_key 拒绝:"
if { psqlq -c "INSERT INTO plans (id, user_id, l1_mode, l2_mode, version, is_active,
                                 investable_monthly_cents, profile_snapshot_json,
                                 l2_allocation_json, emergency_json)
               SELECT gen_random_uuid(), user_id, l1_mode, l2_mode, version, FALSE,
                      0, '{}'::jsonb, '{}'::jsonb, '{}'::jsonb
               FROM plans WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME')
               ORDER BY version DESC LIMIT 1;" 2>&1 || true; } | grep -q "plans_user_version_key"; then
  echo "  ✓ 被 plans_user_version_key 拒绝(版本号用户内唯一)"
else
  echo "  ✗ 重复版本号插入成功 —— 版本约束失效"
  exit 1
fi

say "9. 重新生成入口的预填依据(服务端档案 + 问卷页可直接重算)"
curl -s --noproxy '*' -b "$JAR" "$BASE/profiles/me" | python3 -c '
import json,sys
d=json.load(sys.stdin)["data"]
assert d["questionnaire_completed"] is True, d
assert d["inflow_cents"] == 1500000 and d["expense_fixed_monthly_cents"] == 450000, d
assert d["horizon"] == "y5_10" and d["drawdown_response"] == "hold", d
print("  ✓ 档案完整(收入 15,000 / 固定支出 4,500 / 久期 5-10 年 / 回撤不动)—— 重新进入问卷时由它逐题预填")
'
# 答完的人再进问卷落在步 6(推荐 + 生成方案),「上一步」可回退到任意一步改数字 ——
# 每一步的输入框初值都来自上面那份档案(Wizard 的 initialAnswers)
curl -s --noproxy '*' -b "$JAR" "$WEB/questionnaire" -o "$TMP/q.html"
python3 - "$TMP/q.html" <<'PY'
import pathlib, sys
html = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
checks = {
    "重新进入直接落在可重算的步 6(生成方案)": "生成方案" in html,
    "可回退改任意一步(上一步)": "上一步" in html,
    "步 6 是推荐位(为你推荐)": "为你推荐" in html,
}
for name, ok in checks.items():
    print(f'  {"✓" if ok else "✗"} {name}')
assert all(checks.values()), "问卷页断言失败"
print("✓ 重新生成动线:首页/方案页 → 问卷(预填档案)→ 生成方案 → 版本 +1")
PY

say "10. 应急金已覆盖时的首页摘要(达标口径)"
# 存款 24,000 → 300,000(元),远超应急目标(12 × 8,100 = 97,200),走到「已覆盖」分支
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":5,"inflow_cents":1500000,"expense_fixed_monthly_cents":450000,"savings_cents":30000000,"goal":"wealth"}' \
  -o /dev/null
curl -s --noproxy '*' -b "$JAR" -X POST "$BASE/plans" -H 'content-type: application/json' \
  -d '{"l1_mode":"four_accounts"}' -o "$TMP/plan-v3.json" -w 'HTTP %{http_code}\n'
curl -s --noproxy '*' -b "$JAR" "$WEB/" -o "$TMP/home-met.html"
python3 - "$TMP/plan-v3.json" "$TMP/home-met.html" <<'PY'
import json, pathlib, sys
plan = json.load(open(sys.argv[1]))["data"]
html = pathlib.Path(sys.argv[2]).read_text(encoding="utf-8")
e = plan["emergency"]
print(f'  应急金:is_met={e["is_met"]} 覆盖={e["coverage_tenths"]/10:.1f} 个月 超出={e["surplus_cents"]/100:,.2f}')
assert e["is_met"] is True, e
assert plan["version"] == 3, plan["version"]
checks = {
    "首页说「已覆盖」而不是「还差」(不是百分比)": "应急金已覆盖" in html and "还差约" not in html,
    "已覆盖带覆盖月数": "个月)" in html,
    "达标后仍有「每月可投资」摘要": "每月可投资" in html,
}
for name, ok in checks.items():
    print(f'  {"✓" if ok else "✗"} {name}')
assert all(checks.values()), "已覆盖形态断言失败"
print("✓ 应急金两个分支(未达标 / 已覆盖)在首页都有正确口径")
PY

printf '\n✓ ticket 06 端到端实测全部通过\n'