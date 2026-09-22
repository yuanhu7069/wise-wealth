# 01 · 域层与数据层：tracking 纯函数 + 迁移 0007 + snapshot_service

Status: open
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

-
