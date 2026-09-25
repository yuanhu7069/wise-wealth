# 06 · 验收收尾与期收口

Status: resolved
Labels: ready-for-agent

> f-plan T6。前置：05 完成。DoD 以 prd-f.md 元信息块为准。

## 步骤

1. AC-1～13 全量走查（桌面 + 375px × 亮/暗；截图存 `.scratch/f-modes/shots/`）；AC-6/7/11 异常路径实机触发
2. 埋点 SQL 查验：`plan_generated`（含 entry=mode_lib 至少 1 条）/ `page_view` p06 各 ≥1 条，金额零入 payload 复核
3. restore-drill 复跑（表结构零变更故无增项，确认既有抽验清单全绿），记录回填 arch-f §7
4. `test-report-f.md` 产出（docs/f/）：AC 回链、arch 基线红线 19 条逐项 ✅、T1/T2 闸门记录、安全审计（cargo/npm audit 复跑）、**零迁移与零 engine 改动声明核对**
5. 期文档收口：prd-f 状态 → 已完成；f-plan §3 收口清单逐项勾；金字塔挂账登记核对（prd-f OUT-003 + arch-f §11）
6. DoD 验收动作（苑问）：P06 用四笔钱真实生成一份方案（版本 +1）+ 标准普尔存疑链路完整走查（卡片 → 详情辟谣 → 手动生成 → 方案页警示条）
7. 期收口提交（沿 D/E 期惯例：feat(ticket 06) 期收口 + docs 提交）

## DoD

- AC-1～13 全绿或挂账有据；check.sh 最终状态全绿
- test-report-f 落盘；生成记录/快照/调整清单三件齐备
- 苑问人工走查通过 + DoD 验收动作完成，期收口

## Comments

- 2026-09-25 · AI · AC 全量走查收口(截图存 shots/,生产模式):AC-1/2/3/4/5/6/7/8/9/12/13 ✅;**AC-10 验证缺口**(库内无 caution 模式,纯函数映射代码审读,首个 caution 模式入库时覆盖);**AC-11 架构说明**(后端整体不可达 → requireSession 统一落登录重定向,B 期既定;「后端活但取数失败」组件与 E 期 AC-16 同构,实机模拟方式待苑问商定)。prd-f AC-6/7 措辞按实机呈现回填(卡内错误条 / 内联过期错误,沿 P03 先例)。
- 2026-09-25 · AI · 埋点 SQL:page_view p06 ×4、plan_generated entry=mode_lib ×2;金额零入载荷正则复核 0 条。restore-drill 复跑 12 项全一致(含 plans 指纹,佐证零迁移)。审计:cargo rsa medium(既有挂账)+ npm js-yaml high ×2(存量传递依赖,零新增;是否 npm audit fix 留苑问拍板)。check.sh 终态 5/5 全绿(144 测试)。
- 2026-09-25 · AI · test-report-f.md 终稿落 docs/f/;prd-f 状态 → **已实现(待苑问走查)**;f-plan 收口清单 5/6 勾(期收口提交留待苑问两项完成后);前端已恢复 dev 模式驻留 3000,后端 target/debug 驻留 8080。
- 2026-09-25 · 苑问 · 页面点开查看,**暂时没看到问题,暂验收通过,后续有问题再说;先收口**。(口头走查;DoD 验收动作的实质——新模式真实生成与存疑链路——已由当日测试生成 v11-v13 与走查截图覆盖)
- 2026-09-25 · AI · 期收口:prd-f → 已完成(暂验收口径注明);f-plan 收口清单全勾;本期闭。挂账移交后续期:gen-types.sh TS7 修法、js-yaml audit fix 拍板、dev hydration 环境问题、AC-10 caution 验证缺口、理财金字塔(OUT-003)。
