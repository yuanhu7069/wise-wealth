# 05 · P08 生成与固化：T0 自查 → 生成 → T1 验收 → 固化

Status: open
Labels: ready-for-agent

> g-plan T5。前置：04 完成。规格：design-g.md 全文；customInstructions 同目录（P08 节）。

## 步骤

1. **生成前授权**：第二次真实 BYOK 调用（预算 1 run），获授权后才开始
2. T0 自查：四件套 + daemon 连通性；复用 `ww-redesign-github` 对话式追加生成 P08（提示词以 3 列对比表格为样：维度成行/模式成列、金额列等宽右对齐、不可行列警示短语、2 列与 3 列两形态、375px 横滑首列固定）；不从零重生成
3. T1·A 逐值比对 + T1·B 自动项（表格 th scope / 横滑 / 五态 / 极端数据：超长模式名、负数、仅 2 列）
4. 过闸后固化：快照 `docs/design/g-knowledge/p08-compare-github.html` + 生成记录补行

## DoD

- T1 全过（自动项）；人眼项按先例并入期末复核
- 快照与生成记录行落盘，sha256 一致
