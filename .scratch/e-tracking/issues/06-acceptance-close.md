# 06 · 验收收尾与期收口

Status: resolved
Labels: ready-for-agent

> e-plan T6。前置：05 完成。DoD 以 prd-e.md 元信息块为准。

## 步骤

1. AC-1～16 全量走查（桌面 + 375px × 亮/暗；截图存 `.scratch/e-tracking/shots/`）；AC-2/15/16 异常路径实机触发
2. 埋点 SQL 查验：snapshot_submit / snapshot_delete / snapshot_skip / export_csv / page_view p05 各 ≥1 条，金额零入 payload 复核
3. restore-drill 增项复跑（抽验清单 + snapshots 行数/金额合计），记录回填 arch-e §7
4. `test-report-e.md` 产出（docs/e/）：AC 回链、arch 基线红线 19 条逐项 ✅、T1/T2 闸门记录、安全审计（cargo/npm audit 复跑）
5. 期文档收口：prd-e 状态 → 已完成；e-plan §3 收口清单逐项勾；两轮真实打卡中当月一条由苑问完成即记 DoD 验收动作部分达成，次月复访后全闭
6. 期收口提交（沿 D 期惯例：feat(ticket 06) 期收口 + docs 提交）

## DoD

- AC-1～16 全绿或挂账有据；check.sh 最终状态全绿
- test-report-e 落盘；生成记录/快照/调整清单三件齐备
- 苑问人工走查通过，期收口

## Comments

- 2026-09-23 · AI · 走查记录:苑问人工走查**通过**(P01 卡两态 → P05 五段 × 亮/暗 × 375px → 录入/覆盖/特殊/删除/导出)。DoD 验收动作:真实打卡 2026-09 一条已落(04:11),次月复访为跨月自然验证。
- 2026-09-23 · AI · restore-drill 增项复跑 12 项全一致;埋点 SQL 查验 8 类事件齐(金额零入载荷);cargo audit 离线仅已知 rsa medium,npm audit 网络受限未跑(零新增前端依赖)。代码评审 10 项发现全修复(`a7df41c`),明细 test-report-e §9。
- 2026-09-23 · AI · 期收口:test-report-e 终稿 + prd-e「已完成」+ e-plan 收口清单全勾。工具链待拍板遗留:gen-types.sh TS7 修法;RISK-E-1 TLS 维持对外前挂账。
