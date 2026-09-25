# 03 · P07 生成与固化：T0 自查 → 生成 → T1 验收 → 固化

Status: open
Labels: ready-for-agent

> g-plan T3。前置：02 完成。规格：design-g.md 全文；customInstructions = `.scratch/g-knowledge/custom-instructions.md`（已落盘）。

## 步骤

1. **生成前授权**：向苑问确认本次为真实 BYOK 调用（预算 1 run，≈5-10 min），获授权后才开始
2. T0 自查：四件套逐项核对 + daemon 连通性（连不上 → 停下报告）
3. 复用 project `ww-redesign-github` **对话式追加**生成 P07（提示词以真实篇目清单为样：4 解读卡各带可信度徽章、考据卡、8 词条小卡网格、3 不可落地卡带警示徽章、文章展开态）；不从零重生成
4. T1·A token 逐值比对（基准 = P01 固化契约）；T1·B 自动项（自包含/焦点/44px/reduced-motion/五态/徽章文字+色/「局限性」不弱化）；人眼项交苑问
5. 过闸后固化：快照 `docs/design/g-knowledge/p07-knowledge-github.html` + `生成记录.md` 补行；globals.css 原则零改动

## DoD

- T1·A 全中 + T1·B 自动项全过；苑问人眼项勾选（或按先例并入期末复核）
- 快照与生成记录行落盘；sha256 记录一致；run id/耗时/四件套可回答「怎么来的」
