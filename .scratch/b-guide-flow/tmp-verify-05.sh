#!/usr/bin/env bash
# ticket 05 端到端实测(curl):金例 A 走完问卷 → 生成方案 → 读回方案,断言五段数据。
# 证据脚本,不是交付物;从 .env 取种子口令,不打印。
set -euo pipefail
cd "$(dirname "$0")"
set -a
# shellcheck disable=SC1091
source ../../.env
set +a

BASE="http://127.0.0.1:8080/api/v1"
JAR=$(mktemp)
trap 'rm -f "$JAR"' EXIT

say() { printf '\n===== %s =====\n' "$1"; }

say "1. 登录(种子账号)"
curl -s --noproxy '*' -c "$JAR" -X POST "$BASE/auth/login" \
  -H 'content-type: application/json' \
  -d "{\"username\":\"$SEED_USERNAME\",\"password\":\"$SEED_PASSWORD\"}" | head -c 200
echo

say "2. 未带 Cookie 生成方案(越权测试,期望 401)"
curl -s --noproxy '*' -o /tmp/plan-noauth.json -w 'HTTP %{http_code} ' -X POST "$BASE/plans" \
  -H 'content-type: application/json' -d '{"l1_mode":"four_accounts"}'
cat /tmp/plan-noauth.json; echo

say "3. 金例 A 问卷分步保存"
# 步 1 久期 5-10 年 / 步 2 回撤「不动」/ 步 3 收入「波动大」/ 步 4 赡养 1 人
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":1,"horizon":"y5_10"}' | head -c 160; echo
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":2,"drawdown_response":"hold"}' | head -c 160; echo
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":3,"income_stability":"volatile"}' | head -c 160; echo
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":4,"has_social_security":true,"has_commercial_insurance":false,"dependents":1}' | head -c 160; echo
# 步 5:月收入 12,000 / 固定支出 4,500 / 存款 24,000 / 目标「财富增值」
curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE/profiles/me/step" -H 'content-type: application/json' \
  -d '{"step":5,"inflow_cents":1200000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' | head -c 200
echo

say "4. 步 6 推荐(金例 A 存款 24,000 > 3×固定支出 13,500 → 主推 fifty_30_20)"
curl -s --noproxy '*' -b "$JAR" "$BASE/modes" | python3 -c '
import json,sys
d=json.load(sys.stdin)["data"]
print("recommended:", d["recommended_id"], "| reason:", d["recommendation_reason"])
print("l2:", d["l2"]["name"], [(c["name"], c["basis_points"]) for c in d["l2"]["classes"]])
'

say "5. 生成方案(four_accounts)"
curl -s --noproxy '*' -b "$JAR" -X POST "$BASE/plans" -H 'content-type: application/json' \
  -d '{"l1_mode":"four_accounts"}' -o /tmp/plan-1.json -w 'HTTP %{http_code}\n'

say "6. 读回当前方案并断言"
curl -s --noproxy '*' -b "$JAR" "$BASE/plans/active" -o /tmp/plan-active.json -w 'HTTP %{http_code}\n'

python3 - <<'PY'
import json
gen = json.load(open("/tmp/plan-1.json"))["data"]
act = json.load(open("/tmp/plan-active.json"))["data"]
p = gen

print("mode:", p["l1_mode_name"], "| version:", p["version"], "| date:", p["created_date"])
print("buckets:")
for b in p["buckets"]:
    print(f'  {b["bucket_id"]:<8} {b["name"]:<8} {b["amount_monthly_cents"]/100:>10,.2f}  target={b["target_cents"]}')
e = p["emergency"]
print("emergency:", {k: e[k] for k in ("months","necessary_monthly_cents","target_cents","gap_cents","monthly_toward_emergency_cents","months_to_fill","coverage_tenths","surplus_cents","is_met")})
print("l2:", p["l2"]["name"], [(c["name"], c["basis_points"]) for c in p["l2"]["classes"]])
print("notices:", p["notices"])

# --- 断言(金例 A:ticket 01 已锁定的四个桶 4,500 / 3,600 / 3,100 / 800) ---
amounts = {b["bucket_id"]: b["amount_monthly_cents"] for b in p["buckets"]}
assert amounts == {"salary": 450000, "spend": 360000, "reserve": 310000, "invest": 80000}, amounts
assert p["l2"]["name"] == "60/40", p["l2"]
assert [c["basis_points"] for c in p["l2"]["classes"]] == [6000, 4000], p["l2"]["classes"]
assert e["months"] == 12, e["months"]                      # 波动大 9 + 赡养 1 人上浮一档 = 12(封顶)
assert e["is_met"] is False
assert e["coverage_tenths"] == 30, e["coverage_tenths"]     # 24,000 / 8,100 = 约 3.0 个月
assert e["months_to_fill"] == 24, e["months_to_fill"]       # 与原型的「约 24 个月补齐」一致
assert not p["notices"], p["notices"]
assert all(b["amount_monthly_cents"] >= 0 for b in p["buckets"])
# 生成与读回是同一份快照
assert act["id"] == p["id"] and act["version"] == p["version"], (act["id"], p["id"])
print("\n✓ 金例 A 五个断言组全部通过(桶金额 / L2 / 应急月数 / 非负 / 生成=读回)")
PY
