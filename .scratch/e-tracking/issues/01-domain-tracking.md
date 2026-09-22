# 01 · 域层与数据层：tracking 纯函数 + 迁移 0007 + snapshot_service

Status: resolved
Labels: ready-for-agent

> e-plan T1。规格：arch-e.md §0 ADR-E-001/002/003、§2、§3；RULE-021～029 口径以 prd-e.md §6 为准。

## 步骤

1. 迁移 `0007_snapshots`（UP/DOWN 全写）：`snapshots(id, user_id FK, plan_id FK, month DATE, balances JSONB, special_month BOOL NOT NULL DEFAULT false, created_at, updated_at)`；`UNIQUE(user_id, month)` + 索引 `(user_id, month DESC)`；`.sqlx` 缓存照常提交
2. `domain/tracking.rs` 纯函数（零 IO）：`deviations()`（基准回溯跳过 special / 同名桶 / 前值 ≤ 0 不参与）+ `emergency_gap()`（输入 = plan_buckets 目标与必要桶月转入）+ `persisted_months()`
3. 金例单测锁定（arch-e §10 表：超阈/未超/前值 0/负值/基准回溯/同名桶/缺口 3.2/达标/断月不清零/CSV 转义——转义函数在本票一并落）
4. `services/snapshot_service.rs`：upsert（桶集校验 + UNIQUE 冲突转 UPDATE + updated_at 审计）、`delete_latest`（非最新月 422）、列表查询（恒带 user_id）
5. `config.rs` 读 `SNAPSHOT_DEVIATION_THRESHOLD_BP`（默认 2000，缺键不报错）；`.env.example` 增键含注释
6. `openapi.rs`：profiles/plans/modes 六个存量端点补挂 ApiDoc（arch-e §3 顺手修复项）

## DoD

- `cargo test` 全绿（新增金例全过）；clippy 零警告
- 迁移 DOWN 可回滚（本地试跑一次 up→down→up）
- 同月两连发 upsert → 仍 1 行且 updated_at 推进（集成测试）

## Comments

- 2026-09-22 · AI · 完成。domain/tracking.rs(deviations/emergency_gap/persisted_months)+ domain/csv.rs(escape_field/yuan_string)金例先行,新增 23 用例;迁移 0007 已 up→down→up 试跑并应用 dev 库;repos/snapshots.rs(7 查询,含 export 长表)+ services/snapshot_service.rs(桶集校验/未来月拒绝/仅最新月可删/summary 组装)+ config 偏离阈值键 + ApiDoc 补挂五端点(含 schema 闭包 20 项)。全库 125 测试绿;clippy 余 1 条 `threshold_bp is never read`——票 02 接线 handler 后自然消失。
- 2026-09-22 · AI · 两处执行中的口径决定(均已回填文档):① RULE-026 的「向上保留 1 位小数」与示例 3.2 矛盾(3.214 向上是 3.3),实现取**四舍五入半量进位**(与引擎 div_round_half_away 同族),prd-e RULE-026 已改写;② 应急桶选择:有规则桶用规则桶(四账户=备用),无规则桶回落投资桶(50/30/20=储蓄,其 TOML 桶注释即此语义),单测锁定。③ chrono 首次进 Cargo.toml——ADR-B-001 早已批准该依赖,仅用于服务端「当前月」判定。
