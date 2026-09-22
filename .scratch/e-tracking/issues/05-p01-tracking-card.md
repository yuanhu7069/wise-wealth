# 05 · P01 追踪卡：摘要形态改造 + 打卡入口

Status: open
Labels: ready-for-agent

> e-plan T5。前置：04 完成。**代码级改动，不回炉生成**（基线：落地后真源 = 代码库）。

## 步骤

1. P01 Server 取数扩展：快照 summary（最新月是否已打卡 / 应急缺口 / persisted_months）
2. 摘要形态追加追踪卡：当月未打卡 → 「本月还没打卡」+「去打卡」主按钮（→ `/tracking`）；已打卡 → 最近快照月份 + 缺口月数摘要行
3. 空态形态（无方案）不变；卡片加载失败静默隐藏（不阻塞 P01 主体，prd-e §8.3）
4. 骨架态对齐既有 P01 骨架模式
5. `page_view p01` 既有埋点不受影响；不新增事件

## DoD

- `bash scripts/check.sh` 全绿
- 双形态 × 亮/暗 × 375px 走查过（含追踪卡各态：未打卡/已打卡/骨架/隐藏）
- 未回炉生成核对：本次无任何 OpenDesign 调用

## Comments

-
