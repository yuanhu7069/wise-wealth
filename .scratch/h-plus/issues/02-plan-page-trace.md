# 02 · P04 方案页增量：「为什么」入口 + plus 链条 / free 占位双态

Status: resolved
Labels: ready-for-agent

> h-plan T2。规格：arch-h.md §0 ADR-H-001/004、§1、§3；prd-h §4.2 AC-1～6、§8.3 双态矩阵。
> **代码级增量，不回炉生成**（落地后真源 = 代码库，沿 G 期 P06 先例）。

## 步骤

1. api-types 手工移植：`PlanView` 增 `trace_count: number` 与 `traces?: TraceView[]`（可选——free 响应无此键）；以 `curl /api/v1/openapi.json` 实际输出为准逐字段对照（ADR-G-004 立场），移植记录写本票 Comments
2. P04 应急金状态区增「为什么」收展入口：aria-expanded + 关联区域（P06 收展先例）；数据随方案响应一次性下发，展开**零额外请求**
3. plus 态渲染：链条 3-4 节点纵排，每节点 = 规则名 + 输出值 + 一句话依据（`rationale` 原样渲染，不增删改，RULE-047）；金额等宽千分位右对齐
4. free 态渲染（无 `traces` 键时）：Plus 徽章 + 模糊占位（链条形状可见、内容不可读、模糊区 aria-hidden）+ 一句示例 + 「N 步推理，Plus 可展开」计数；**无任何解锁按钮**（OUT-001）
5. 会话过期：内联过期错误 + 重登链接（`from=` 回跳，沿 F/G 先例，AC-5）
6. `bash scripts/check.sh` 全绿；Token 门禁过

## DoD

- `check.sh` 全绿；`git diff` 确认 globals.css 零改动
- AC-1～6 走查过：free/plus 双态 × 375px/桌面 × 亮/暗；curl 复验 free 响应无 `traces` 键
- 让位发生档案（构造：收入不足）链条含让位节点；常规档案无该节点、不占位（AC-2）

## Comments

- 2026-09-27 · AI · 完成。api-types 手工移植(以新二进制 `/api/v1/openapi.json` 实际输出为准):PlanView +trace_count/traces,新增 TraceView / TraceUnitView / ReviewView / BucketDeltaView / EmergencyConvergenceView / Envelope_ReviewView 六 schema;**沿用 G 期惯例只移植 components 段,paths/operations 段缺失属 gen-types 拍板项的既有欠账,不在本期扩大**。UI:`plan/trace-panel.tsx` 客户端岛(收展 aria-expanded + aria-controls;plus 链条节点 = 规则标签 + rationale → 输出值,连接线 border-l 画;free = blur 占位 + Plus 徽章 + N 步计数 + 通栏示例句,**无任何解锁动作**,OUT-001);文案进 state.ts TRACE_RULE_COPY / TRACE_COPY(唯一来源立场;未知 rule_id 兜底显示 id);挂载点 = 一、分配总览卡应急金区下方(链条解释的正是那几个数字)。
- 2026-09-27 · AI · **Token 门禁拦一次**:首版连接线用了 left-[7px] / top-[11px] / w-px 裸 px,改 token 类(border-l + 圆点/轨道同心几何)后过。check.sh 全绿(biome 仅 1 条 warning = G 期 knowledge 页遗留,非本期文件)。闸门双向 curl 实测(真实 dev 库):free 无 traces 键 + trace_count=3 → set-tier plus → 全链下发(rationale 与方案冻结一致,含赡养上浮文案)→ 还原 free 无键;**tier 已还原 free,8081 冒烟服务已停**。375px / 亮暗人眼走查交苑问(票 05 统一)。
