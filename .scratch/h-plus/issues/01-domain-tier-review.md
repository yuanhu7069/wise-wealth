# 01 · 域与服务层：engine 让位 trace + tier 闸门 + tracking::review + review 端点

Status: ready-for-agent
Labels: ready-for-agent

> h-plan T1。规格：arch-h.md §0 ADR-H-001/002/003、§2、§4、§5、§10；RULE-045～052 口径以 prd-h.md §6 为准。

## 步骤

1. `domain/engine.rs`：**唯一引擎改动**——在既有让位分支追加 `Trace { rule_id: "safety_first_yield", output: 让位后投资桶月转入（分）, unit: Cents, rationale: 让位次序与结果一句话 }`，仅当让位实际发生（投资桶归零或规则桶缩减）时产出；**金额计算路径、函数签名、既有输出零改动**（ADR-H-002）
2. tier 闸门（ADR-H-001）：service 层读会话用户当前 tier；`PlanView` 增 `trace_count: usize`（恒下发）+ `traces: Vec<TraceView>`（仅 plus 序列化，free 下 `skip_serializing_if` 字段整体不出现）；tier 未知值按 free 处理 + 启动告警
3. `domain/tracking.rs`（或既有 tracking 域文件）增 `review(user, quarter)`：自然季度边界（01-01/04-01/07-01/10-01 起）、进程内聚合快照 + active 方案应急目标；**复用 E 期应急进度口径函数，不新写第二套**；特殊月计 `persisted_months`、不入环比序列；仅 1 条快照 → delta=0 + `single_snapshot: true`；空季 → 空集（ADR-H-003）
4. `GET /api/v1/tracking/review?quarter=YYYY-QN`（缺省当前季）：dto 校验 `^\d{4}-Q[1-4]$` + 年份 ≥2026，非法 → 422；空季 200 空集；**端点不读 tier**（复盘免费全量，RULE-051）；纯读、零写库
5. `PageId` 枚举 +P09；埋点白名单扩
6. `scripts/set-tier.sh`：`set-tier.sh <username> <free|plus>`，psql UPDATE，连接参数走 `.env`；README 增一行用法；**不挂 API**
7. 金例单测（arch-h §10 全表）：让位档案 traces 长度 4 + `safety_first_yield` 字段断言；常规档案长度恒 3；**既有金例全量回归**（solve 金额断言零改动、全绿）；preview 对账回归（含 traces 同源）；tier 闸门双向（free 响应无 `traces` 键 + `trace_count` 正确；plus 全量；tier 变更后同方案响应变化）；review 聚合金例（3 条正常 / 含特殊月 / 单条 / 空季 / 收敛数值金例 / `2026-Q5`→422 / `abc`→422 / 无 Cookie→401）

## DoD

- `cargo test` 全绿（**既有金例一条不改、全过**为硬门）；clippy 零警告
- `git diff` 审查：engine.rs 仅让位 trace 追加（diff 记录进票 05 审计）；无任何写库调用进入 review/preview 路径
- free 响应无链条内容的单测断言绿（RISK-H-1 结构性证明）
- 启动日志正常；`.env.example` 零新增；零迁移（`ls server/migrations` 无 0008）

## Comments

-
