# 08 · P02 登录页 + loading/404（铺开收尾）

Status: needs-triage
Labels: ready-for-agent

> 前置：07 完成。生成前打招呼；胜者 project 内迭代。

## 步骤

1. 生成 P02 登录页（含 `from` 回跳提示态）
2. 路由级 loading 骨架 / not-found 404 按新 token 体系重做（小件，可与 P02 同轮）
3. T1 → 固化 → 落地 → T2，流程同 03–05

## DoD

- P02 + loading/404 过 T2，check.sh 全绿，生成记录补一行
## Comments

- 2026-09-21 · 苑问 · 改批量授权:本页生成无需再打招呼;T1 人眼项与 T2 走查并入期末统一复核(ticket 09)。AI 连续推进:生成 → T1 自动项 → 固化 → 落地 → T2 代码级。
- 2026-09-21 · AI · P02 生成已发(批量授权内):run `f64bd6af-0344-491d-bff6-4964d0dcc113`,requestId `bef4f756-d219-4603-b7b1-3a5106a46689`,胜者 project 对话式追加。提示词含登录卡真实文案(标题/副标题/过期条/双输入/错误样本)、44px 输入与 focus 蓝边、通栏绿主按钮 pending 态、极端数据核验。loading/404 为 app 路由件,按已固化 token 体系重做,不单独生成。
- 2026-09-21 · AI · 生成完成(4.0 min,`p02-login.html` 507 行,stable-prompt 缓存命中,四页齐)。T1 自动项 ✅(10/10 token、自包含、焦点×3、44px×6、autocomplete×3、aria-describedby、reduced-motion)。
- 2026-09-21 · AI · 固化 `docs/design/od-redesign/p02-login-github.html` + 生成记录补行。落地为轻触式:登录卡结构经换肤已与产物同构(6px 平面卡/44px 输入/通栏绿主按钮/role=alert 错误关联),仅过期提示条对齐产物(attention-subtle 底 + warning 图标 + role=status)。loading.tsx(P01 时已随产物形状)/ not-found.tsx(纯 token 类,自动换肤)无需再改。check.sh 全绿 → T2 代码级通过。
- 2026-09-21 · AI · P04/P03/P02 全部生成+落地完成,发出统一复核邀请(预览链接 + 复核清单),人眼验收全部集中在复核环节。
