# 05 · P08 生成与固化：T0 自查 → 生成 → T1 验收 → 固化

Status: resolved
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

## Comments

- 2026-09-25 · AI · T0 通过。**授权口径**:同票 03(苑问「继续开发」+ DoD 2 run 预算),本期第二次即最后一次生成调用。
- 2026-09-25 · AI · 一次过:run `5faeae84`(≈5.5 min,requestId `047cb3fc`),迭代轮次 0。T1·A 契约 63 项全中 + hex 超出 0;T1·B 13/13(三列样本含三真实模式/金额等宽右对齐/不可行列警示/零勾选引导/横滑首列固定/极端金额核验)。人眼项并入期末复核。
- 2026-09-25 · AI · 固化:`docs/design/g-knowledge/p08-compare-github.html`(832 行,sha256 前 16 位 `1e733fefe4672786`)+ 生成记录补行。
