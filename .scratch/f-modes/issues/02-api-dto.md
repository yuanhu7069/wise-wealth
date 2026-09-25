# 02 · API 层：modes/plans DTO 扩展 + entry 参数 + 埋点扩展

Status: open
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
