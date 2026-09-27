# 架构实例：智策理财 H 期 · Plus 起步——推理链展开与季度复盘（arch-h.md）

> 关联规范：docs/base_line/基线-arch-Web域.md v0.2 / 基线-design-OpenDesign-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260927-01 智策理财 H 期推理链展开与季度复盘（prd-h.md）
> 前序实例：docs/g/arch-g.md（继承 b/arch-v2.md、e/arch-e.md、f/arch-f.md；**增量式**：已锁定内容不重述，仅记 H 期新增与变更）
> 数据库：外部 PostgreSQL dev 库（连接参数见 `.env`；无 TLS —— RISK-E-1 挂账延续）
> 上游总纲：docs/智策理财_PRD_V1.1.md（§5.1 推理链 P1、§4.4.3 T4 季度复盘、§10.1 分层边界）；领域词汇：根 CONTEXT.md「Plus 能力」节 +「追踪」节（本期新增三术语）

---

## 0. 实例 ADR（H 期项目级决策）

> ADR-A/B/E/F/G 各期决策继续有效；本期无推翻。G 期「零引擎改动」是期约束，本期按 ADR-H-002 有界解除（仅输出追加）。

### ADR-H-001：Plus 闸门在服务侧——`users.tier` 实时过滤，响应形状随分层变化

【状态】已接受（2026-09-27，grilling Q2 拍板）
【上下文】`users.tier` 自迁移 0002 预留（`free` 默认），从未启用；产品 PRD §2.1「一套内核，两套暴露」要求闸门在内核侧；无支付通道；免费态需要「知道自己缺什么」的转化素材。
【决策】① 启用 `users.tier`，取值 `free | plus`；② 闸门唯一位置 = Rust API/service 层：active 方案响应（`PlanView`）增 `trace_count: usize`（恒下发）与 `traces: Vec<TraceView>`（**仅 plus 序列化**，free 下 `skip_serializing_if` 使字段整体不出现）；③ 实时语义：service 层按会话用户**当前** tier 过滤，与方案版本无关——置 plus 后刷新即得全链，无需重新生成（RULE-046）；④ 置 plus = `scripts/set-tier.sh`（psql UPDATE 包一脚本，用法入 README），产品内无购买/升级路径（OUT-001）；⑤ 本期不做 tier 缓存（单用户直查，微秒级）。
【后果】正面：分层语义真实（free 响应体无内容可偷，RISK-H-1 的结构性消解）；接支付时只换 tier 的写入方，闸门零改动。负面：响应形状随分层变化——前端类型 `traces?: TraceView[]` 可选，P04 需双态渲染（free 占位 / plus 全链）；同 URL 不同用户看到不同深度，测试矩阵 ×2。

### ADR-H-002：引擎推理链最小增补——`safety_first_yield` 让位节点，金额语义零变

【状态】已接受（2026-09-27，grilling 范围拍板）
【上下文】引擎现有 3 条 trace（`emergency_fund_months` / `emergency_fund_target` / `emergency_fund_pacing`），让位链（投资桶归零 → 缩减规则桶 → 缺口提示）只走 `notices` 无 trace——恰是产品 PRD §5.1 示例链最关键的一环；B 期 engine.rs 注释已预告「将来开放推理链时不必回填历史」。G 期「零引擎改动」是期约束，本期为补齐链条完整性有界解除。
【决策】在 `engine::solve` 既有让位分支**追加** 1 条 `Trace { rule_id: "safety_first_yield", output: 让位后投资桶月转入（分）, unit: Cents, rationale: 让位次序与结果一句话 }`——仅当让位实际发生（投资桶被归零或规则桶被缩减）时产出；**solve 的金额计算路径、函数签名、既有输出（buckets/l2/emergency/notices）零变**，traces 长度 3 → 4（让位发生时）。新增金例：让位档案断言第 4 节点存在且字段正确，非让位档案断言长度恒 3。
【后果】正面：链条覆盖完整（AC-2）；preview 与生成同源，对账自动覆盖新节点。负面：engine.rs 出现本期唯一改动点——完成门挂全量金例 + preview 对账回归（RISK-H-2，任一红即回退增补，AC-2 降级为「三条既有规则」并登记变更记录）。

### ADR-H-003：复盘聚合在服务侧域层——`GET /tracking/review`，自然季度，零迁移

【状态】已接受（2026-09-27，grilling Q3 拍板）
【上下文】复盘三件内容的原料（snapshots、plans 应急目标）已在库；特殊月/达标/缺口口径已在 E 期域层落地。前端聚合会复制业务口径造第二真源；季度语义已拍板自然季度。
【决策】① 新域函数 `tracking::review(user, quarter) -> ReviewView`：进程内聚合该季快照（≤3 条）+ active 方案应急目标，口径**复用 E 期既有应急进度函数**（备用桶余额 vs 应急目标），不新写第二套；② 端点 `GET /api/v1/tracking/review?quarter=YYYY-QN`（缺省 = 服务器时钟当前季）；响应 `{quarter, persisted_months, buckets: [{bucket_id, name, quarter_start_cents, latest_cents, delta_cents}], emergency: {target_cents, start_gap_cents, current_gap_cents, avg_monthly_convergence_cents, months_to_goal}, special_months: ["YYYY-MM"]}`；③ 特殊月：计入 `persisted_months`，其月份不入环比序列（RULE-050）；仅 1 条快照 → `delta_cents=0` 且响应带 `single_snapshot: true`；空季 → 200 空集（非 404）；④ quarter 非法格式 → 422；⑤ **零数据库迁移**（连续第三期）。
【后果】正面：口径单点（E 期函数复用）；P09 单请求渲染；跨季同比届时加字段不破坏形状。负面：「季内变化」以本季首末快照为端点（非日历季初零点）——与「还差多少」的直觉一致但需在 RULE-050 与 UI 标注说清（AC-10 单点形态）。

### ADR-H-004：P09 新页生成 + P04/P05 代码级增量；推理链数据随方案一次性下发

【状态】已接受（2026-09-27，grilling Q1/Q2 拍板，沿 ADR-G-003 先例）
【上下文】产品 PRD §5.0「Plus 不是独立页面，而是每个界面的展开能力」→ 推理链挂在 P04；复盘是报告形态的新信息架构 → 独立页值得一次生成预算；P04/P05 上期（或更早）已过验收。
【决策】`P09 /tracking/review` 走 Open Design 生成（1 run）；P04 增量（「为什么」入口 + 链条/占位双态收展区）与 P05 增量（复盘入口）为**代码级改动**（落地后真源 = 代码库，同 E 期 P01 卡、G 期 P06 先例）。推理链数据随 active 方案响应一次性下发——展开/收起为纯前端交互，零额外请求。
【后果】正面：生成预算 1 run 边界清晰；展开零延迟。负面：方案响应体积增 ~1KB（4 节点短文本，可忽略）；free 态下该体积只含条数。

---

## 1. 页面与路由映射（H 期增量）

| 页面ID | 页面名称 | Next 路由 | 组件类型 | 备注 |
|--------|---------|-----------|---------|------|
| P09 | 季度复盘页（新） | `/tracking/review?quarter=YYYY-QN`（缺省当前季） | Server 壳（解析 quarter + 调 review）+ 三卡（server 渲染） | 季度切换器为 Client 件（产合法值跳转）；生成走 design-h 三道闸门 |
| P04 | 方案页（改造） | `/plan` | + Client 收展区（aria-expanded，P06 先例） | 「为什么」入口 + plus 链条 / free 占位；**代码级，不回炉** |
| P05 | 追踪页（改造） | `/tracking` | + 复盘入口（卡/按钮，Link） | **代码级** |
| P01-P03/P06-P08 | - | - | - | 零改动 |

鉴权边界：`/tracking/review` 进 requireSession 重定向链（`/login?from=<path>`）；服务端强制仍在 Rust 中间件（新端点落 JWT，白名单**零变更**）。P04 既有登录前置不变。

## 2. 核心实体与数据模型（H 期增量）

**零数据库迁移**（连续第三期）：分层字段已在 0002、推理链已随 plans 冻结、快照已在 0007。

### 2.1 分层字段启用（ADR-H-001）

| 项 | 值 |
|------|------|
| 字段 | `users.tier TEXT NOT NULL DEFAULT 'free'`（既有，0002 COMMENT 已声明预留语义） |
| 取值 | `free` / `plus`（service 层枚举，未知值按 free 处理 + 启动告警） |
| 写入方 | 本期仅 `scripts/set-tier.sh`（运维）；产品内无写入路径 |
| 读取点 | plans/review service 层（会话用户当前 tier） |

### 2.2 复盘聚合视图（ADR-H-003，域层 ReviewView）

| 字段 | 类型 | 口径 |
|------|------|------|
| `quarter` | `YYYY-QN` | 本次聚合的季度 |
| `persisted_months` | usize | 全部快照自然月累计（非季内；RULE-050①） |
| `buckets[]` | 桶级季内变化 | `quarter_start_cents` = 本季首条快照余额；`latest_cents` = 本季末条；`delta_cents` = 末−初；单条 → 0 + `single_snapshot` |
| `emergency{}` | 缺口收敛 | target（active 方案）；start/current gap（E 期口径：应急目标 − 备用桶余额）；avg 收敛 = (start−current) ÷ 参与环比月数；`months_to_goal` = current ÷ avg 线性外推 |
| `special_months[]` | `YYYY-MM` 列表 | 计入坚持、不入环比（RULE-050③） |

## 3. 需求 → 能力映射（RULE-045 起 → 代码落点）

| RULE | 后端实现位置 | 前端实现位置 |
|------|-------------|-------------|
| RULE-045（Plus 闸门服务侧） | plans service 按 tier 过滤；`PlanView.traces` skip_serializing_if | P04 双态渲染；free 永远拿不到内容字段 |
| RULE-046（分层实时生效） | service 读会话用户当前 tier，不读方案冻结层 | 置 plus 后刷新即得全链 |
| RULE-047（如实呈现） | 引擎 trace 为唯一链条来源 | 原样渲染节点，不增删改 |
| RULE-048（覆盖范围） | `engine.rs` 让位分支追加 `safety_first_yield`（ADR-H-002） | 链条 3-4 节点纵排 |
| RULE-049（自然季度） | `tracking::review` quarter 解析与边界（Q 起 01-01/04-01/07-01/10-01） | 季度切换器只产合法值 |
| RULE-050（三件口径） | `tracking::review` 聚合（复用 E 期应急进度函数；特殊月剔除环比） | 三卡渲染 + 特殊月标注 + 单点形态 |
| RULE-051（复盘免费全量） | review 端点**不读 tier** | P09 无分层分支 |
| RULE-052（金额不入日志埋点） | tracing 纪律（延续） | page_view p09 仅 page_id |

顺手修复：无。

## 4. API 清单（H 期新增/变更，均挂 `/api/v1`）

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| GET | `/plans/active`（既有） | 响应增字段：`trace_count`（恒有）+ `traces`（仅 plus 序列化，ADR-H-001） | JWT | - | 既有 PlanView + 上述两字段；free 响应无 `traces` 键 |
| GET | `/tracking/review` | 复盘聚合（ADR-H-003） | JWT | `?quarter=YYYY-QN`（缺省当前季） | ReviewView；空季 200 空集；非法 quarter → 422 |
| POST | `/analytics/events` | 既有端点，白名单扩 | JWT | page_view + p09 | 不变 |

`scripts/set-tier.sh`（新增，运维）：`set-tier.sh <username> <free|plus>`，内部 psql UPDATE；**不挂 API**。
codegen：DTO 变更后手工移植 api-types（ADR-G-004 立场延续；gen-types.sh TS7 拍板项仍挂账）。

## 5. 权限校验点清单（H 期增量）

| 校验点 | 位置 | 校验内容 | 越权/未认证时行为 |
|--------|------|---------|------------------|
| 中间件全路由校验 | `api/middleware/auth.rs` | review 端点落 JWT（白名单零变更） | 401 UNAUTHORIZED 信封 |
| tier 过滤 | plans service | plus → traces 全量；free → 仅 trace_count（字段不出现） | 结构性不可越权（无请求形态） |
| quarter 格式 | dto 校验 | `^\d{4}-Q[1-4]$` 且年份合理（≥2026） | 422 VALIDATION_ERROR |
| review 无写路径 | `api/v1/tracking.rs::review` | 纯读聚合 | - |
| 页面鉴权 | requireSession | /tracking/review 进重定向链 | 302 → /login?from= |
| 埋点白名单 | `analytics_service.rs` | PageId 枚举 +P09 | 422 |

越权/降级测试：review 无 Cookie → 401；free 会话方案响应 `traces` 键不存在（单测断言）；`quarter=2026-Q5` → 422；`quarter=abc` → 422。

## 6. 外部依赖与集成（H 期增量）

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| Open Design daemon 0.22.1（127.0.0.1:7456） | P09 界面生成（**生成期一次性 ×1 run**，运行时零依赖） | 连不上 → 停下报告苑问，不手搓（基线 §2.2） | BYOK（用户侧） |
| 其余 | **无新增 crate / 中间件 / 环境变量键** | - | - |

## 7. 数据与迁移（H 期增量）

- **零迁移**（连续第三期）；restore-drill 复跑无增项，记录回填 test-report-h
- `scripts/set-tier.sh` 进 Git（运维脚本，无凭证，连接参数走 `.env`）
- 红线 15 自查：不触碰资金类数据 ✅

## 8. 安全要点（H 期增量）

- **分层闸门是安全边界**：free 响应无 `traces` 键为单测锁定项；评审清单加一条「plans/review 响应路径禁止在 free 分支拼接 traces」
- 复盘纯读聚合，无写库路径；金额不入日志与埋点（tracing 只带 request_id + 路由名）
- 输入面：quarter 格式全量 DTO 校验；tier 未知值按 free 降级 + 告警（fail-safe 方向 = 给得少）

## 9. 部署与启动（增量）

拓扑不变。`start.sh` 不改（零迁移）；`check.sh` 不改（新增用例自动纳入）。新增 `scripts/set-tier.sh`；README 增「置 plus」一行（苑问自用操作）。

## 10. 测试计划（基线 §14.3 模式）

**域/服务层单测（金例锁定）**：

| 用例 | 断言 |
|------|------|
| 让位 trace 金例（收入不足档案） | traces 长度 4；`safety_first_yield` 节点 output = 0（投资桶归零）且 rationale 非空 |
| 非让位金例（常规档案） | traces 长度恒 3；无 `safety_first_yield` |
| **既有金例全量回归** | solve 金额断言逐条不动、全绿（ADR-H-002 完成门） |
| preview 对账回归 | preview.solution == 生成路径 solve（含 traces 同源，ADR-G-002 单测扩展） |
| tier 闸门 | 同一方案：free 响应无 `traces` 键且 `trace_count=3`；plus 响应 traces 全量；tier 更改后同方案响应变化（实时性） |
| review 聚合金例 | 3 条快照季（正常 delta）；含特殊月（剔除环比但计坚持）；单条快照（delta=0 + single_snapshot）；空季（200 空集）；缺口收敛与 months_to_goal 数值金例 |
| review 边界 | `2026-Q5`/`abc`/`2025-Q1`（早于首快照且有语义）→ 422 或空集（按格式错/合法空季分派）；无 Cookie → 401 |

**前端手工清单**：AC-1～14 走查（375px + 桌面 × 亮/暗 × free/plus 双态）；五态对照 prd-h §8.3；Token 门禁（check-tokens.sh）。

**设计闸门**：P09 按 design-h.md 过 T0/T1/T2；T1 自动项记 test-report-h。

**备份恢复**：restore-drill 复跑（无增项）。

## 11. H 期交付物清单与验收顺序

1. `server/`：engine.rs 让位 trace（唯一引擎改动）+ plans service tier 闸门 + PlanView 增字段 + `tracking::review` + review 端点 + PageId +P09 + `scripts/set-tier.sh`（**零迁移**）
2. `web/`：P04 增量（为什么入口 + 双态）+ P05 入口 + P09 新页（生成落地）+ api-types 手工移植
3. `docs/design/`：P09 固化快照 + `生成记录.md` 追加一行
4. `docs/h/`：test-report-h.md（另批）；`.scratch/h-plus/` 工单（同批）
5. 验收顺序：让位 trace 金例 + 既有金例全量回归绿 → preview 对账回归绿 → tier 闸门单测绿 → review 聚合金例绿 → 鉴权链路（无 Cookie 401）→ T0 自查 → P09 生成 → T1 → 固化 → P09 落地 + T2 → P04 增量 + P05 入口 → 双态端到端走查（free 占位 → set-tier → 刷新见全链 → 复盘三卡 → 切季 → 空态）→ 埋点 SQL → restore-drill → 苑问 DoD 验收动作

**存量挂账核对**：RISK-E-1（TLS）/ 金字塔（OUT-007 延续）/ gen-types.sh TS7（拍板项延续）/ js-yaml audit fix / dev hydration 环境问题 / AC-10 caution 缺口——全部延续；新增无。
