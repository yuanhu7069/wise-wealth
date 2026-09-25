# F 期设计文档 · 模式库页（P06 生成落地）

> 基线：`docs/base_line/基线-design-OpenDesign-Web域.md`（Open Design MCP 生成型 v0.1，2026-09-20 生效）
> prd / arch：`docs/f/prd-f.md`（REQ-20260925-01）/ `docs/f/arch-f.md` —— 本期为功能期，三件套配套
> D/E 期先例：`docs/d/design-d.md`、`docs/e/design-e.md`（生成型基线第 2、3 期；其固化契约即本期视觉基准）

## 1 范围

| 页 | 处置 | 说明 |
|---|---|---|
| **P06 模式库页（`/modes`）** | **走生成**（本期唯一生成对象） | 卡片列表 + 展开详情（出处 + 桶概览）+ CTA；prd-f §8.3 五态矩阵为 T1 完整性判据 |
| P01 首页 / P04 方案页 | 代码级改造（入口链接 + 可信度提示条） | 基线明文：落地后真源 = 代码库，改动直接改代码，**不回炉生成** |
| P02/P03/P05 / loading / 404 | 零改动 | - |

## 2 T0 · 生成前参数四件套（生成前必须齐备，未齐不开生成）

| 参数 | 值 | 依据 |
|---|---|---|
| design system | `github`（od://design-systems/github/DESIGN.md，run digest `5b03d62f…91`，同 D/E 期五页） | D 期对比法胜者，已固化进 `web/src/app/globals.css` |
| skill | `redesign-existing-projects` | 沿用 D/E 期——skill 承载「与既有项目同体系」的生成形态，P06 虽是新页，仍须落进既有视觉体系 |
| fidelity | high-fidelity | 同 D/E 期 |
| customInstructions | `.scratch/f-modes/custom-instructions.md`（**已落盘进 Git**：D 期定稿全文 + F 期 P06 追加节） | 基线红线 6 |

**迭代载体**：复用胜者 project `ww-redesign-github`，对话式追加（基线 §3.1 复用同一 project；红线 5 禁止每轮新建）。生成前照例只做 daemon 连通性验证（127.0.0.1:7456）；连不上 → 停下报告，不手搓、不换来源。每次生成是一次真实 BYOK 调用，不批量重跑；生成前需苑问授权（一次）。

## 3 生成预算与流程

- **预算：1 个新页 × 对话式追加 1 run**（参照 D/E 期同类耗时 ≈ 4-10 min）；产物不满意走**对话式修改**迭代，不从零重生成。
- 流程：追加生成 → 立即 T1 → 过闸后固化（§6）→ 落地 P06 → T2。
- 生成提示词内容以 4 模式列表为样例（含标准普尔 disputed 卡与四笔钱 verified 卡的对照），卡片必须同时给出**收起态**与**展开态**（customInstructions 追加节）。

## 4 T1 · 草稿验收（未过不得落地；红线 9）

**A · 一致性（自动比对）**：产物 token 值逐项对照 P01 固化时的 GitHub 契约值（`docs/design/od-redesign/p01-home-github.html` + `globals.css` 为基准）——对不上即漂移，判失败。

**B · 完整性（风格无关六项 + 本期人眼项）**：

| # | 判据 | P06 具体化 |
|---|---|---|
| 1 | 五态齐全 | 列表骨架 / 防御空态（「模式库暂时为空」）/ 错误条 + 重试 / CTA 生成中 loading；卡片收起 ↔ 展开两态齐备 |
| 2 | 极端数据不破版 | 模式名 / tagline / 出处超长截断折行、4 卡列表、桌面双列与 375px 单列两布局；**本页无金额输入**（无金额极端用例） |
| 3 | 中文渲染 | 标题无负字距、细体加重、行高放宽（D 期既定派生规则）；出处书名号排版正确 |
| 4 | 自包含 | 零外链、不依赖 daemon |
| 5 | 无障碍 | 对比度达标、焦点环、44px 触控、reduced-motion；**徽章文字 + 颜色双通道**（不只靠色） |
| 6 | 数值展示 | 桶概览百分比对齐；**本页警示语义**：disputed 用徽章 + 辟谣文案承载，不做整卡红色遮蔽（customInstructions 追加节） |

执行者：自动项 AI 跑，人眼项（亮/暗 × 375px 逐态）苑问按表勾。

## 5 T2 · 落地验收

- 判据不变：实现与固化产物的偏离必须能在调整清单找到，清单外一律照产物执行。
- 调整清单落 `.scratch/f-modes/spec.md` §4（模板沿 E 期模式）；高频调整项（中文字重/字距档、CJK 字体栈、lucide 图标）沿 D/E 期 §4.1 结论直接复用，无需重新论证。

## 6 固化（P0，生成后立即）

| 动作 | 位置 | 要求 |
|---|---|---|
| 搬进代码库 | `globals.css` **原则上零改动**（契约已固化；若产物确需新 token → 先进调整清单论证，默认拒绝发明新值） | P06 组件层对齐产物 |
| 留快照 | `docs/design/f-modes/p06-modes-github.html` | 单文件自包含副本 |
| 记录元数据 | `docs/design/生成记录.md` 追加一行 | 四件套 / run id / 耗时 / 产物 sha256 前 16 位 |

固化完成后，P06 的设计真源 = 代码库；后续改动直接改代码。

## 7 工件索引（随执行回填）

- 产物快照：`docs/design/f-modes/`（待生成）
- 生成记录：`docs/design/生成记录.md`（追加行）
- customInstructions：`.scratch/f-modes/custom-instructions.md`（已落盘）
- 调整清单 / 工单：`.scratch/f-modes/`（同批）

## 8 一句话收口

> 参数四件套已锁（github / redesign-existing-projects / high-fidelity / custom-instructions 已入库）→ 一页一 run 对话式追加 → T1 逐值过闸 → 固化进 Git → 落地后真源在代码库。
