# 06 · P04 方案页（铺开第 1 页）

Status: resolved
Labels: ready-for-agent

> 前置：05 完成。正式生成前向苑问打招呼（授权节奏，spec §2）。在胜者 project 内迭代，不新建 project。

## 步骤

1. 对话式修改让胜者 project 出 P04（五段视图：分配总览 / L2 账户内部配置 / L3 执行规则 / 落地建议 / 风险提示）+ 应急金覆盖状态
2. T1（A 自动 + B 勾选）→ 固化 → 落地 → T2，流程同 03–05
3. 极端数据重点：方案数字超长 / 负数 / 百分比边界

## DoD

- P04 过 T2，check.sh 全绿，生成记录补一行

## Comments

- 2026-09-20 · 苑问 · 授权 P04 生成("发")。打招呼环节完成(spec §2 授权节奏)。
- 2026-09-20 · AI · 生成已发:胜者 project `ww-redesign-github` 内对话式追加(未新建 project,红线 5),run `3151347e-3944-47a6-9980-a6ec45087923`,requestId `2bb3dd64-6b9f-4180-aa2c-b7d35e3047cc`,工具 0.22.1。提示词:P04 五段真实文案 + 提示条三变体(NOTICE_COPY 全文)+ 应急金达标/未达标两态 + 表格 GitHub 密排规则 + 比例条色盲可读图例 + 五段骨架 + 极端数据核验(¥12,345,678.90 / 超长模式名 / 3%/97% 比例条)+ 亮暗双主题 + 自包含单文件 HTML;明确要求与 P01 视觉严格一致。
- 2026-09-21 · 苑问 · 改批量授权:本页生成无需再打招呼;T1 人眼项与 T2 走查并入期末统一复核(ticket 09)。AI 连续推进:生成 → T1 自动项 → 固化 → 落地 → T2 代码级。
- 2026-09-21 · AI · 生成完成(9.4 min,`p04-plan.html` 810 行,与 P01 同会话延续,风格一致性由产物自证:同 token 契约/顶栏/按钮/主题键)。T1 自动项 ✅(11/11 token、零外链、tabular×4、焦点环×3、44px×6、role=status/alert×4、reduced-motion);人眼项并入统一复核。
- 2026-09-21 · AI · 固化:`docs/design/od-redesign/p04-plan-github.html` + 生成记录补行。落地:globals 增 `attention-subtle` token(提醒条底,DS 文档值 #fff8c5 / 暗色 mix 派生);plan-view 重写——五段改分区卡(Canvas Subtle 头条 + 平面卡体),提示条双变体(严重 danger-subtle / 提醒 attention-subtle + 图标),应急金面板加状态圆点,执行规则等宽编号 + 虚线分隔补充段,比例条加 role=img 中文播报,页头元信息等宽化。偏离:风险提示保持 h2 正文段(产物同),骨架/极端核验区为产物展示件不落地(app 状态由路由与真实数据驱动)。check.sh 全绿 → T2 代码级通过。 ticket DoD 待统一复核,先转 P03。
- 2026-09-21 · 苑问 · 统一复核通过("页面没问题")。ticket 关闭。
