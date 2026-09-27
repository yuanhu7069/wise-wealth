# 01 · 域与服务层：engine 让位 trace + tier 闸门 + tracking::review + review 端点

Status: resolved
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
## Comments

- 2026-09-27 · AI · 完成。① engine.rs:仅让位分支追加 `yielded` 标记 + `safety_first_yield` trace(三段 rationale:收入不足 / 缩减规则桶至 N 分 / 无规则桶仅归零;output = 投资桶让位后金额)——diff 审计确认金额路径零改动;**发现:50/30/20 无余量桶,该模式本就不会归零让位,天然不产出第四节点**(与既有 notice 行为一致,非缺陷)。② tier:`domain/tier.rs`(parse 未知值降级 Free)+ `users::tier_of`(每请求现读,RULE-046 实时)+ `PlanView.trace_count`(恒下发)/`traces`(仅 plus 序列化,free 无键);`to_view` 增 tier 参数,生成/读取两端点同口径。③ 复盘:`tracking::quarter_review` 纯函数(端点=本季最早/最新**非特殊**快照;同名桶才可比;月均按环比步数摊,<2 个有效点不给;缺口扩大如实负值、不外推)+ `snapshots::list_range_asc` + service 编排(应急口径复用 E 期 `emergency_status_of`/`emergency_bucket_id`,零新口径)+ `GET /api/v1/tracking/review`(缺省当前季,422/200空集;**不读 tier**,RULE-051)+ `PageId::P09`。④ `scripts/set-tier.sh`:psql 变量绑定(`:'var'`)防注入;UPDATE 0 行也退 0 的坑用显式存在性查询兜住;README 快速开始补一行。⑤ sqlx 离线缓存 +2 条(随代码入库)。
- 2026-09-27 · AI · DoD:cargo test **169 全绿**(+15 金例:让位 4 节点/恒 3 节点/闸门双向/复盘 8 例/tier 解析);既有金例零改动全过;preview 对账单测绿;clippy -D warnings 零警告;启动冒烟(8081):health ok、openapi 含 tracking/review、无 Cookie → 401;零迁移(0007 止)、`.env.example` 零新增。附带重构:`api::tests_support::state()` 从 mod tests 提出复用(纯测试夹具搬家)。
