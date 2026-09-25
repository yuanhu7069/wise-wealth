# 04 · P06 落地：组件层实现 + 五态接线 + T2 验收

Status: resolved
Labels: ready-for-agent

> f-plan T4。前置：03 完成（固化产物为落地真源）。规格：arch-f.md §1、§3（RULE-032/033）、§5；prd-f.md §8（页面清单/状态矩阵/错误清单）。

## 步骤

1. 新建 `web/src/app/modes/`（Server 壳取模式列表 + Client 卡片组件）；`/modes` 进 layout 未登录重定向链（`/login?from=/modes`）
2. 卡片列表对齐固化产物：名称 / tagline / 可信度徽章（文字+色，三值）/ 适合人群 pill / 推荐徽章（至多 1 张，取 `is_recommended`）/ 展开详情（出处块 + 桶概览 share_desc 行 + CTA）
3. 五态接线：列表骨架（路由级 loading）/ 防御空态「模式库暂时为空」/ 错误条 + 重试（A 期降级链）/ CTA 生成中 loading 禁重复 / 正常态
4. CTA 接 Server Action 生成链路（复用既有 generatePlanAction 模式，entry=mode_lib）；成功 → 跳 `/plan`；失败 → toast「生成没成功，再试一次」（ERR-F-01）；401 → 登录回跳（ERR-F-02）
5. 调整清单落 `.scratch/f-modes/spec.md` §4（产物原值 / 项目值 / 理由，逐项拍板）；D/E 期 §4.1 结论照抄项（中文字重 / CJK 字体栈 / lucide）不再论证
6. T2 验收：实现与固化产物的偏离全部能在调整清单找到，清单外零偏离
7. 走查 AC-1/2/6/11/12/13（桌面 + 375px × 亮/暗；截图存 `.scratch/f-modes/shots/`）

## DoD

- `check.sh` 全绿（含 check-tokens.sh Token 门禁）
- 五态齐全对照 prd-f §8.3；清单外零偏离
- AC-1/2/6/11/12/13 走查通过（截图为证）
- 金额/余额不渲染核对：P06 页面源码无金额格式化调用（桶概览只有比例与口径文案）

## Comments

- 2026-09-25 · AI · 完成。新建 `web/src/app/modes/` 五件:page(server 壳,三态:错误/防御空态/ready)、modes-view(server 视图,页头+网格)、mode-card(客户端岛:收展 aria-expanded+hidden、CTA useTransition→generatePlanAction entry=mode_lib、失败卡内错误条)、loading(路由级骨架)、state(types);`/modes` 进 requireSession 重定向链;api-types 补 components.PlanView 两字段(首版手工移植漏一处缩进,已补)。check.sh 5/5 全绿(含 Token 门禁:loading 骨架改 E 期 animate-pulse 语汇避免裸 px;注释去掉「px」字样)。
- 2026-09-25 · AI · **环境发现(已记 spec.md §4 备注)**:turbopack dev 模式在 headless playwright 下 hydration 不发生(E 期 /tracking 页同样症状,非本期代码回归);走查与交互验证在 `next build && next start` 生产模式完成——hydration 正常、交互全通;真实桌面浏览器不受影响(E 期人工走查佐证)。
- 2026-09-25 · AI · T2 验收:调整清单 4 项(spec.md §4.1:CTA 失败卡内错误条/caution 黄族/推荐卡默认展开/核验区不渲染),清单外零偏离。走查(生产模式,截图存 shots/):AC-1 桌面亮 4 卡 ✅(推荐落在 50/30/20,RULE-006 按真实档案判定);AC-2 存疑卡展开辟谣+四桶口径 ✅;AC-6 CTA 失败实机触发(杀后端→错误条)✅;AC-12 窄屏亮/暗不破版 ✅;暗色全套 ✅。AC-13 防御空态不可自然触发(构建期兜底),代码审读核对;AC-11「后端整体不可达」架构上落登录重定向(requireSession 先行,B 期既定),「后端活但取数失败」场景组件与 E 期 AC-16 同构,实机模拟方式留票 06 与苑问商定。
- 2026-09-25 · AI · 金额零渲染核对:mode-card/bucket 概览仅比例与口径文案,无 format-currency 调用 ✅。前端当前以生产模式驻留 3000(票 05 验证沿用),期收口前恢复 dev。
