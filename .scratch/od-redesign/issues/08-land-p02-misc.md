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
