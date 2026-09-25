# G 期设计文档 · 知识库页与对比页（P07/P08 生成落地）

> 基线：`docs/base_line/基线-design-OpenDesign-Web域.md`（Open Design MCP 生成型 v0.1，2026-09-20 生效）
> prd / arch：`docs/g/prd-g.md`（REQ-20260925-02）/ `docs/g/arch-g.md` —— 本期为功能期，三件套配套
> D/E/F 期先例：`docs/d/design-d.md`、`docs/e/design-e.md`、`docs/f/design-f.md`（其固化契约即本期视觉基准）

## 1 范围

| 页 | 处置 | 说明 |
|---|---|---|
| **P07 知识库页（`/knowledge`）** | **走生成**（1 run） | 四板块分区 + 文章阅读态；prd-g §8.3 五态矩阵为 T1 完整性判据 |
| **P08 对比页（`/modes/compare`）** | **走生成**（1 run） | 参数表格（含试算金额列，375px 横滑首列固定） |
| P06 模式库页 | 代码级增量（勾选 + 链接） | 基线明文：落地后真源 = 代码库，**不回炉生成** |
| P01-P05 | 零改动 | - |

## 2 T0 · 生成前参数四件套（生成前必须齐备，未齐不开生成）

| 参数 | 值 | 依据 |
|---|---|---|
| design system | `github`（od://design-systems/github/DESIGN.md，run digest `5b03d62f…91`，同 D/E/F 期六页） | D 期对比法胜者，已固化进 `web/src/app/globals.css` |
| skill | `redesign-existing-projects` | 沿用 D/E/F 期——新页仍须落进既有视觉体系 |
| fidelity | high-fidelity | 同前 |
| customInstructions | `.scratch/g-knowledge/custom-instructions.md`（**已落盘进 Git**：D 期定稿全文 + G 期追加节） | 基线红线 6 |

**迭代载体**：复用胜者 project `ww-redesign-github`，对话式追加（红线 5 禁止每轮新建）。生成前照例只做 daemon 连通性验证；连不上 → 停下报告。每次生成是一次真实 BYOK 调用，**P07 与 P08 各需苑问授权一次**，不批量重跑。

## 3 生成预算与流程

- **预算：2 新页 × 各对话式追加 1 run**（参照 D/E/F 期同类耗时 ≈ 4-10 min/run）；产物不满意走**对话式修改**，不从零重生成。
- 流程：P07 追加生成 → T1 → 固化 → P07 落地 → T2；再 P08 追加生成 → T1 → 固化 → P08 落地 → T2。
- 生成提示词内容：P07 以真实篇目清单为样例（4 解读卡含各自可信度徽章 + 8 词条小卡 + 3 不可落地卡带警示徽章）；P08 以 3 列对比表格为样例（含金额列与不可行列警示短语两种形态）。

## 4 T1 · 草稿验收（未过不得落地；红线 9）

**A · 一致性（自动比对）**：产物 token 值逐项对照 P01 固化时的 GitHub 契约值（`docs/design/od-redesign/p01-home-github.html` + `globals.css` 为基准）——对不上即漂移，判失败。

**B · 完整性（六项 + 本期人眼项）**：

| # | 判据 | P07/P08 具体化 |
|---|---|---|
| 1 | 五态齐全 | P07：骨架 / 错误条 / 板块空防御 / 正常 / 文章展开态；P08：骨架 / 错误 / 0 勾选引导 / 正常 / 不可行列警示形态 |
| 2 | 极端数据不破版 | 标题超长截断、sections 空段、对比仅 2 列与 3 列两形态、金额超长与负数、375px 横滑 |
| 3 | 中文渲染 | 标题无负字距、细体加重、行高放宽；「局限性」「不建议照搬」不做视觉弱化 |
| 4 | 自包含 | 零外链、不依赖 daemon |
| 5 | 无障碍 | 对比度、焦点环、44px、reduced-motion；表格 th scope 语义 |
| 6 | 数值展示 | 金额等宽千分位右对齐、负数红字（沿红跌绿涨外的一般金额负数红字既定项） |

执行者：自动项 AI 跑，人眼项（亮/暗 × 375px 逐态）苑问按表勾。

## 5 T2 · 落地验收

- 判据不变：实现与固化产物的偏离必须能在调整清单找到（落 `.scratch/g-knowledge/spec.md` §4），清单外一律照产物执行。
- 高频调整项沿 D/E/F 期结论直接复用，无需重新论证。

## 6 固化（P0，生成后立即）

| 动作 | 位置 | 要求 |
|---|---|---|
| 搬进代码库 | `globals.css` **原则上零改动**（契约已固化；若产物确需新 token → 先进调整清单论证，默认拒绝发明新值） | P07/P08 组件层对齐产物 |
| 留快照 | `docs/design/g-knowledge/p07-knowledge-github.html`、`p08-compare-github.html` | 单文件自包含副本 |
| 记录元数据 | `docs/design/生成记录.md` 各追加一行 | 四件套 / run id / 耗时 / sha256 前 16 位 |

固化完成后，P07/P08 的设计真源 = 代码库；后续改动直接改代码。

## 7 工件索引（随执行回填）

- 产物快照：`docs/design/g-knowledge/`（待生成）
- 生成记录：`docs/design/生成记录.md`（追加两行）
- customInstructions：`.scratch/g-knowledge/custom-instructions.md`（已落盘）
- 调整清单 / 工单：`.scratch/g-knowledge/`（同批）

## 8 一句话收口

> 参数四件套已锁 → 两页各一 run 对话式追加 → T1 逐值过闸 → 固化进 Git → 落地后真源在代码库；P06 增量只改代码不回炉。
