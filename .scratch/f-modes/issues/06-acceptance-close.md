# 06 · 验收收尾与期收口

Status: open
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
