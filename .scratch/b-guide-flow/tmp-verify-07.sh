#!/usr/bin/env bash
# ticket 07 端到端实测:埋点链路(5 事件)。证据脚本,不是交付物。
#
# 覆盖 ticket 07 的五个验收项:
#   1. 5 类事件均可入库(页面访问 / 问卷开始 / 每步完成带步号 / 问卷完成 / 方案生成)
#   2. 方案生成事件载荷含模式 id 与版本号
#   3. 写入失败不阻断主流程 —— 本脚本验不到(需要让库坏掉),由 cargo test 的
#      `services::analytics_service::tests::写入失败不阻断主流程` 断言
#   4. 事件载荷不含任何金额数值(全表键白名单 + 数值上界)
#   5. 可用 SQL 直接查验事件类型分布与各步到达情况
#   另:问卷开始是**一次性**事件(prd-v1 §9.5「首次进入」)—— 刷新页面不该把开始率刷上去
#
# 两半都验:后端三类事件经 curl 走真实接口;页面触达与问卷开始**只能**由前端
# 服务端渲染触发,故直接请求 :3000 的 P01/P03/P04(带会话 Cookie),再回库对账。
#
# 口令从 .env 取,不打印。**本脚本会清掉种子账号的档案与方案行、并清空埋点表** ——
# 问卷开始事件需要「还没有草稿」的真实状态,这些本就是可重建的验证数据
# (scripts/seed-user.sh 幂等重置)。脚本结束时该账号停在「问卷答全 + 第 2 版方案」。
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
FAIL=0
trap 'rm -rf "$JAR"' EXIT

say() { printf '\n===== %s =====\n' "$1"; }
psqlq() { psql "$DB" -v ON_ERROR_STOP=1 -tAq "$@"; }
count() { psqlq -c "SELECT count(*) FROM analytics_events WHERE $1;"; }
expect() { # expect <说明> <实际> <期望>
  if [ "$2" = "$3" ]; then
    echo "  ✓ $1 = $2"
  else
    echo "  ✗ $1:期望 $3,实际 $2"
    FAIL=1
  fi
}
# 前端触达走 after():响应发出后才发请求,断言不能查得太早 —— 轮询到期望值或超时
wait_count() { # wait_count <说明> <where 子句> <期望>
  local desc="$1" where="$2" want="$3" got=""
  for _ in $(seq 1 24); do
    got=$(count "$where")
    [ "$got" = "$want" ] && break
    sleep 0.25
  done
  expect "$desc" "$got" "$want"
}
get() { curl -s --noproxy '*' -b "$JAR" "$@"; }
put() { curl -s --noproxy '*' -b "$JAR" -X PUT "$BASE$1" -H 'content-type: application/json' -d "$2"; }
post() { curl -s --noproxy '*' -b "$JAR" -X POST "$BASE$1" -H 'content-type: application/json' -d "$2"; }
webhtml() { curl -s --noproxy '*' -b "$JAR" -o /dev/null -w '%{http_code}' "$WEB$1"; }

say "0. 前置:后端与前端在线"
curl -s --noproxy '*' -o /dev/null -w '后端 /health → HTTP %{http_code}\n' "$BASE/health"
curl -s --noproxy '*' -o /dev/null -w '前端 /login → HTTP %{http_code}\n' "$WEB/login"

say "1. 登录(种子账号)"
curl -s --noproxy '*' -c "$JAR" -X POST "$BASE/auth/login" \
  -H 'content-type: application/json' \
  -d "{\"username\":\"$SEED_USERNAME\",\"password\":\"$SEED_PASSWORD\"}" | head -c 80
echo

say "2. 重置到「还没答问卷、没有方案」,并清空埋点表"
psqlq -c "DELETE FROM analytics_events;"
psqlq -c "DELETE FROM plan_buckets WHERE plan_id IN (
            SELECT p.id FROM plans p JOIN users u ON u.id = p.user_id
            WHERE u.username = '$SEED_USERNAME');" >/dev/null
psqlq -c "DELETE FROM plans WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME');" >/dev/null
psqlq -c "DELETE FROM profiles WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME');" >/dev/null
expect "埋点表已清空" "$(count 'TRUE')" "0"

say "3. P01 服务端渲染 → page_view(p01)"
echo "  首页 HTTP $(webhtml '/')"
wait_count "page_view/p01" "event_type='page_view' AND payload_json->>'page_id'='p01'" "1"

say "4. 无方案时访问 P04 → 跳回首页,不记 p04"
code=$(curl -s --noproxy '*' -b "$JAR" -o /dev/null -w '%{http_code}' "$WEB/plan")
echo "  /plan HTTP $code(307 = 无方案跳首页)"
expect "尚未有 page_view/p04" "$(count "event_type='page_view' AND payload_json->>'page_id'='p04'")" "0"

say "5. P03 首次进入(无草稿)→ page_view(p03) + questionnaire_start"
echo "  问卷页 HTTP $(webhtml '/questionnaire')"
wait_count "page_view/p03" "event_type='page_view' AND payload_json->>'page_id'='p03'" "1"
wait_count "questionnaire_start" "event_type='questionnaire_start'" "1"

say "6. 步 1 保存成功 → questionnaire_step_completed{step:1}"
put "/profiles/me/step" '{"step":1,"horizon":"y5_10"}' | head -c 60
echo
expect "step_completed/step=1" "$(count "event_type='questionnaire_step_completed' AND payload_json->>'step'='1'")" "1"

say "7. 有草稿后再进 P03 → 只记触达,不再记问卷开始"
echo "  问卷页 HTTP $(webhtml '/questionnaire')"
wait_count "page_view/p03 累计" "event_type='page_view' AND payload_json->>'page_id'='p03'" "2"
expect "questionnaire_start 仍为" "$(count "event_type='questionnaire_start'")" "1"

say "8. 答完步 2-5 → 每步一条 + 问卷完成恰一条"
put "/profiles/me/step" '{"step":2,"drawdown_response":"hold"}' >/dev/null
put "/profiles/me/step" '{"step":3,"income_stability":"volatile"}' >/dev/null
put "/profiles/me/step" '{"step":4,"dependents":1,"has_social_security":true}' >/dev/null
put "/profiles/me/step" '{"step":5,"inflow_cents":1200000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' | head -c 60
echo
# 每步一条,步号 1-5 各一次(第 1 步在第 6 节已记)
arrived=$(psqlq -c "SELECT string_agg(DISTINCT payload_json->>'step', ',' ORDER BY payload_json->>'step') FROM analytics_events WHERE event_type='questionnaire_step_completed';")
expect "到达的步号" "$arrived" "1,2,3,4,5"
expect "步事件总数" "$(count "event_type='questionnaire_step_completed'")" "5"
expect "questionnaire_completed" "$(count "event_type='questionnaire_completed'")" "1"

say "9. 重复保存步 5(重新生成前改数字)→ 不再记第二条问卷完成"
put "/profiles/me/step" '{"step":5,"inflow_cents":1500000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' >/dev/null
expect "questionnaire_completed 仍为" "$(count "event_type='questionnaire_completed'")" "1"

say "10. 生成方案 → plan_generated{l1_mode, plan_version};P04 渲染 → page_view(p04)"
post "/plans" '{"l1_mode":"four_accounts"}' | head -c 80
echo
expect "plan_generated v1 载荷" \
  "$(psqlq -c "SELECT (payload_json - 'plan_version')::text FROM analytics_events WHERE event_type='plan_generated' AND payload_json->>'plan_version'='1';")" \
  '{"l1_mode": "four_accounts"}'
echo "  方案页 HTTP $(webhtml '/plan')"
wait_count "page_view/p04" "event_type='page_view' AND payload_json->>'page_id'='p04'" "1"

say "11. 重新生成 → 第二条 plan_generated 带新版本号"
post "/plans" '{"l1_mode":"four_accounts"}' >/dev/null
expect "plan_generated 版本号集" \
  "$(psqlq -c "SELECT string_agg(payload_json->>'plan_version', ',' ORDER BY payload_json->>'plan_version') FROM analytics_events WHERE event_type='plan_generated';")" \
  "1,2"

say "12. SQL 查验:事件类型分布(验收动作)"
psqlq -c "SELECT event_type, count(*) AS 条数 FROM analytics_events GROUP BY event_type ORDER BY 2 DESC, 1;"
types=$(psqlq -c "SELECT count(DISTINCT event_type) FROM analytics_events;")
expect "事件类型覆盖" "$types" "5"

say "13. 载荷只含枚举与步号,无任何金额"
# 键白名单:每个事件类型只允许出现这几个键
bad=$(psqlq -c "
  SELECT count(*) FROM analytics_events e
  WHERE EXISTS (
    SELECT 1 FROM jsonb_object_keys(e.payload_json) AS k
    WHERE k NOT IN ('page_id','step','l1_mode','plan_version')
  );")
expect "白名单外的键" "$bad" "0"
# 数值上界:载荷里唯一的数值是步号与版本号(都 < 100)
big=$(psqlq -c "
  SELECT count(*) FROM analytics_events e
  WHERE EXISTS (
    SELECT 1 FROM jsonb_each_text(e.payload_json) AS kv
    WHERE kv.value ~ '^[0-9]+$' AND kv.value::bigint >= 100
  );")
expect "可疑的大数值" "$big" "0"
# 反向证据:同一批数据里方案表确实有金额(说明「埋点里没有金额」不是碰巧,是被挡住了)
expect "当前方案的桶金额(对照)" \
  "$(psqlq -c "SELECT count(*) FROM plan_buckets b JOIN plans p ON p.id = b.plan_id WHERE p.is_active AND b.amount_monthly_cents > 0;")" "4"

say "14. 白名单拒收与鉴权边界"
code=$(curl -s --noproxy '*' -b "$JAR" -o /tmp/ww-e1.json -w '%{http_code}' -X POST "$BASE/analytics/events" \
  -H 'content-type: application/json' -d '{"event":"plan_generated"}')
echo "  客户端上报 plan_generated → HTTP $code $(cat /tmp/ww-e1.json)"
expect "拒收状态码" "$code" "422"
code=$(curl -s --noproxy '*' -b "$JAR" -o /dev/null -w '%{http_code}' -X POST "$BASE/analytics/events" \
  -H 'content-type: application/json' -d '{"event":"page_view"}')
expect "page_view 缺 page_id" "$code" "422"
code=$(curl -s --noproxy '*' -b "$JAR" -o /dev/null -w '%{http_code}' -X POST "$BASE/analytics/events" \
  -H 'content-type: application/json' -d '{"event":"page_view","page_id":"p99"}')
expect "未知页面 id" "$code" "422"
code=$(curl -s --noproxy '*' -b "$JAR" -o /dev/null -w '%{http_code}' -X POST "$BASE/analytics/events" \
  -H 'content-type: application/json' -d '{"event":"questionnaire_start","page_id":"p03"}')
expect "问卷开始携带 page_id" "$code" "422"
code=$(curl -s --noproxy '*' -o /dev/null -w '%{http_code}' -X POST "$BASE/analytics/events" \
  -H 'content-type: application/json' -d '{"event":"page_view","page_id":"p01"}')
expect "无 Cookie 上报" "$code" "401"
expect "拒收的请求没有落库" "$(count "event_type='plan_generated' AND payload_json = '{}'::jsonb")" "0"

say "15. 一次性事件:清空档案后再造一次「无草稿进入」,不再记第二条问卷开始"
psqlq -c "DELETE FROM profiles WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME');" >/dev/null
echo "  档案已清空(等价于「问卷还没开始过」)"
echo "  问卷页 HTTP $(webhtml '/questionnaire')"
sleep 2
expect "questionnaire_start 仍为" "$(count "event_type='questionnaire_start'")" "1"
expect "该次进入的触达已记下" "$(count "event_type='page_view' AND payload_json->>'page_id'='p03'")" "3"
# 把问卷答回去,让账号停在「问卷答全」的可重跑状态
put "/profiles/me/step" '{"step":1,"horizon":"y5_10"}' >/dev/null
put "/profiles/me/step" '{"step":2,"drawdown_response":"hold"}' >/dev/null
put "/profiles/me/step" '{"step":3,"income_stability":"volatile"}' >/dev/null
put "/profiles/me/step" '{"step":4,"dependents":1,"has_social_security":true}' >/dev/null
put "/profiles/me/step" '{"step":5,"inflow_cents":1500000,"expense_fixed_monthly_cents":450000,"savings_cents":2400000,"goal":"wealth"}' >/dev/null
expect "档案已恢复答全" \
  "$(psqlq -c "SELECT questionnaire_completed FROM profiles WHERE user_id = (SELECT id FROM users WHERE username = '$SEED_USERNAME');")" "t"

say "结果"
if [ "$FAIL" = 0 ]; then
  echo "✓ ticket 07 验收项全部通过"
else
  echo "✗ 存在未通过项,见上方 ✗"
  exit 1
fi
