# 06 · 验收收尾与期收口

Status: open
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

-
