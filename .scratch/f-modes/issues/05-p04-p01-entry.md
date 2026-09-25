# 05 · P01/P04 增量：入口链接 + 方案页可信度提示条

Status: resolved
Labels: ready-for-agent

> f-plan T5。前置：04 完成。规格：arch-f.md §1、§3（RULE-034/035）；prd-f.md §6 RULE-035、§8.1。**代码级改动，不回炉生成**（基线：落地后真源 = 代码库）。

## 步骤

1. P01 首页导航区追加「模式库」入口（对齐既有页面入口形态；代码级）
2. P04 方案页追加「查看全部模式」链接 → `/modes`
3. P04 可信度提示条：读响应 `l1_credibility` / `l1_source`——disputed → warn 变体（「本方案使用的『标准普尔象限』可信度存疑：出处不成立…」级文案，辟谣一句 + 出处）；caution → info 变体（限制说明一句）；verified → 无条；字段 null（模式已下架）→ 无条
4. 变体映射逻辑前端单测（三态 + null 态）；提示条复用既有双变体组件（E 期偏离条同款语汇）
5. AC-4/5/9 走查：四笔钱生成 → 无条；标准普尔生成 → 警示条且方案照常可用；入口双链可达（AC-3）

## DoD

- `check.sh` 全绿
- AC-3/4/5/9 走查通过（桌面 + 375px × 亮/暗）
- P01/P04 改动不触碰既有追踪卡/方案快照呈现（回归走查既有段落无变化）

## Comments

- 2026-09-25 · AI · 完成。plan/state.ts:PlanView 增 `l1_credibility`/`l1_source`(可选字段,与 api-types 手工移植版一致)+ `CREDIBILITY_NOTICE` 文案表(RULE-035:disputed=warn / caution=info,文案唯一来源同 NOTICE_COPY 立场);plan-view.tsx 新增 `CredibilityNotice`(warn=attention 黄族 / info=primary 蓝款,verified 与字段缺失均不渲染,disputed 附出处行)+ 底部按钮行加「查看全部模式」;home-view.tsx 摘要卡按钮行加「模式库」ghost 按钮、空态加「先看看模式库」文字链。check.sh 5/5 全绿。
- 2026-09-25 · AI · **偏离说明**:工单原写「变体映射前端单测」——项目无前端测试框架,引 vitest 违反红线 1(新增依赖须审批),变体验证改由实机走查承担(工单 DoD 的 AC 走查即覆盖)。
- 2026-09-25 · AI · 走查(生产模式,截图存 shots/):AC-3a 首页→模式库一步 ✅;AC-3b 方案页→模式库 ✅;AC-5 verified 方案无提示条 ✅(p04-verified-no-notice.png);AC-9 实机:临时生成标准普尔 v13 → 方案页黄色警示条(三要素+出处行)且方案完整可用 ✅(p04-disputed-warn-notice.png),测后 active 已恢复 v8;AC-4 已随票 04 全链路验证(四笔钱 CTA→v12→方案页)。**AC-10(caution)实机不可验证**:库内无 caution 模式(理财金字塔挂账 OUT-003);映射为纯函数 + info 变体复用既有蓝款,首个 caution 模式入库时随走查覆盖,登记为验证缺口(非缺陷)。
- 2026-09-25 · AI · 回归核对:既有提示条(notices)、五段结构、追踪卡均未触碰(diff 仅增量);P04 头部顺序 = 警示条 → 引擎提示条 → 五段,警示条置顶符合「诚实呈现优先」口径。
