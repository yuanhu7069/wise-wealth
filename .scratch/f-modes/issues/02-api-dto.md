# 02 · API 层：modes/plans DTO 扩展 + entry 参数 + 埋点扩展

Status: resolved
Labels: ready-for-agent

> f-plan T2。前置：01 完成。规格：arch-f.md §0 ADR-F-002/004、§3、§4、§5；§9.5 埋点口径以 prd-f.md 为准。

## 步骤

1. `GET /modes` 响应扩展：每项增 `source?: string` 与 `buckets: [{name, purpose, share_desc}]`；`share_desc` 服务端字符串化（fixed_expenses →「按月固定支出划入」/ pct →「每月收入的 N%」/ rule(emergency_fund) →「按应急金节奏划入」/ remainder →「收入结余全部划入」）——前端零解读（ADR-F-004）
2. `POST /plans` 请求体增可选 `entry: "questionnaire" | "mode_lib"`（缺省 questionnaire；未知枚举 422）；`plan_generated` 埋点 payload 增 `entry`
3. `GET /plans/active`（及方案详情响应）增 `l1_credibility` / `l1_source`：读取时由 ModeLibrary 按 l1_mode 解析，模式下架 → null（ADR-F-002，零迁移）
4. 埋点白名单：PageId 枚举 +P06（客户端 `page_view` p06）
5. 重跑 `scripts/gen-types.sh`，`api-types.ts` 零手改；DTO 校验用例（entry 枚举 422 / 无 Cookie 401 ×2 / credibility 解析与降级 null）进集成测试

## DoD

- `cargo test` 全绿；clippy 零警告
- `gen-types.sh` 产物含三个扩展字段；curl 实测三端点响应符合 arch-f §4 表
- 埋点：curl 生成（entry=mode_lib）后 `analytics_events` 出现带 entry 的 `plan_generated` 行
- 确认无新迁移文件、无新 crate、`.env.example` 零新增

## Comments

- 2026-09-25 · AI · 完成。`GET /modes` 增 `source` + `buckets`(share_desc 服务端字符串化,`BucketSpec::share_desc()`,ADR-F-004);`POST /plans` 增可选 `entry` 枚举(questionnaire/mode_lib,缺省 questionnaire 兼容老客户端);`plan_generated` 埋点载荷带 entry(白名单测试同步放行);PageId 枚举 +P06;`PlanView` 增 `l1_credibility` / `l1_source`(读取时解析,模式已下架 → null,ADR-F-002)。cargo test 144 全绿 + clippy 零警告;check.sh 5/5 全绿。零迁移、零 crate、`.env` 零新增,已核对。
- 2026-09-25 · AI · **工具链偏离(需苑问知悉)**:`gen-types.sh` 因 openapi-typescript 与 TS7 不兼容不可用(E 期票 06 已挂账的遗留)。本次以**新构建后端的真实 openapi.json** 为准,按生成器格式手工移植 5 个 schema 差异(ModeCardView / BucketOverviewView / PlanView / GeneratePlanRequest / PlanEntry)进 `api-types.ts`,tsc 校验通过;**「gen-types.sh TS7 修法」建议列入期收口拍板项**。
- 2026-09-25 · AI · DoD 实测:GET /modes 返回 4 卡,标准普尔卡带存疑出处与逐桶 share_desc(10%/20%/30%/40% 全对);curl entry=mode_lib 生成 four_pots v11,`analytics_events` 出现 `payload->>'entry' = 'mode_lib'` 行;entry=elsewhere → 422;无 Cookie → 401;**admin active 已恢复 v8 four_accounts**(测试版本 v11 保留为历史版本)。
