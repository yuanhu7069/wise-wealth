# E 期设计文档 · 追踪模块（P05 生成落地）

> 基线：`docs/base_line/基线-design-OpenDesign-Web域.md`（Open Design MCP 生成型 v0.1，2026-09-20 生效）
> prd / arch：`docs/e/prd-e.md`（REQ-20260922-01）/ `docs/e/arch-e.md` —— 本期为功能期，三件套配套
> D 期先例：`docs/d/design-d.md`（生成型基线首期，四页重设计；其固化契约即本期视觉基准）

## 1 范围

| 页 | 处置 | 说明 |
|---|---|---|
| **P05 追踪页（`/tracking`）** | **走生成**（本期唯一生成对象） | 五段纵排：录入卡 / 距离感进度 / 偏离提示条 / 历史列表 / 导出区；prd-e §8.3 五态矩阵为 T1 完整性判据 |
| P01 首页 | 代码级改造（摘要形态 + 追踪卡） | 基线明文：落地后真源 = 代码库，改动直接改代码，**不回炉生成** |
| P02/P03/P04 / loading / 404 | 零改动 | - |

## 2 T0 · 生成前参数四件套（生成前必须齐备，未齐不开生成）

| 参数 | 值 | 依据 |
|---|---|---|
| design system | `github`（od://design-systems/github/DESIGN.md，run digest `5b03d62f…91`，同 D 期四页） | D 期对比法胜者，已固化进 `web/src/app/globals.css` |
| skill | `redesign-existing-projects` | 沿用 D 期——skill 承载「与既有项目同体系」的生成形态，P05 虽是新页，仍须落进既有视觉体系 |
| fidelity | high-fidelity | 同 D 期 |
| customInstructions | `.scratch/e-tracking/custom-instructions.md`（**已落盘进 Git**：D 期定稿全文 + E 期 P05 追加节） | 基线红线 6 |

**迭代载体**：复用 D 期胜者 project `ww-redesign-github`，对话式追加（基线 §3.1 复用同一 project；红线 5 禁止每轮新建）。生成前照例只做 daemon 连通性验证（127.0.0.1:7456）；连不上 → 停下报告，不手搓、不换来源。每次生成是一次真实 BYOK 调用，不批量重跑。

## 3 生成预算与流程

- **预算：1 个新页 × 对话式追加 1 run**（参照 D 期同类耗时 ≈ 4-10 min）；产物不满意走**对话式修改**迭代，不从零重生成。
- 流程：追加生成 → 立即 T1 → 过闸后固化（§6）→ 落地 P05 → T2。
- 生成提示词内容以录入四账户（4 桶）为样例，但 P05 必须给出 3 桶（50/30/20）与 4 桶两种布局（customInstructions 追加节）。

## 4 T1 · 草稿验收（未过不得落地；红线 9）

**A · 一致性（自动比对）**：产物 token 值逐项对照 P01 固化时的 17 项 GitHub 契约值（`docs/design/od-redesign/p01-home-github.html` + `globals.css` 为基准）——对不上即漂移，判失败。

**B · 完整性（风格无关六项 + 本期人眼项）**：

| # | 判据 | P05 具体化 |
|---|---|---|
| 1 | 五态齐全 | 录入卡错误态（输入保留）/ 历史空态（首月引导）/ 无方案空态 / 骨架 / 成功确认条；偏离条「触发/不触发」两态 |
| 2 | 极端数据不破版 | 金额超长（¥12,345,678.90）、负数红字、缺口 0.0 个月、历史 0 条与 120 条、**桶数 3 与 4 两布局** |
| 3 | 中文渲染 | 标题无负字距、细体加重、行高放宽（D 期既定派生规则） |
| 4 | 自包含 | 零外链、不依赖 daemon |
| 5 | 无障碍 | 对比度达标、焦点环、44px 触控、reduced-motion |
| 6 | 数值展示 | 等宽 tabular、千分位、右对齐；**红跌绿涨语义**（D 期调整清单既定项，生成若相反必须在调整清单纠正） |

执行者：自动项 AI 跑，人眼项（亮/暗 × 375px 逐态）苑问按表勾。

## 5 T2 · 落地验收

- 判据不变：实现与固化产物的偏离必须能在调整清单找到，清单外一律照产物执行。
- 调整清单模板与暗色派生规则沿 D 期（`.scratch/od-redesign/spec.md` §4 模式），本期落 `.scratch/e-tracking/spec.md`（另批工单时建）。
- 预期高频调整项（预警，非授权）：中文字重/字距档、CJK 字体栈、lucide 图标替代——均沿 D 期 §4.1 结论直接复用，无需重新论证。

## 6 固化（P0，生成后立即）

| 动作 | 位置 | 要求 |
|---|---|---|
| 搬进代码库 | `globals.css` **原则上零改动**（契约已固化；若产物确需新 token → 先进调整清单论证，默认拒绝发明新值） | P05 组件层对齐产物 |
| 留快照 | `docs/design/e-tracking/p05-tracking-github.html` | 单文件自包含副本 |
| 记录元数据 | `docs/design/生成记录.md` 追加一行 | 四件套 / run id / 耗时 / 产物 sha256 前 16 位 |

固化完成后，P05 的设计真源 = 代码库；后续改动直接改代码。

## 7 工件索引（随执行回填）

- 产物快照：`docs/design/e-tracking/`（待生成）
- 生成记录：`docs/design/生成记录.md`（追加行）
- customInstructions：`.scratch/e-tracking/custom-instructions.md`（已落盘）
- 调整清单 / 工单：`.scratch/e-tracking/`（另批）

## 8 一句话收口

> 参数四件套已锁（github / redesign-existing-projects / high-fidelity / custom-instructions 已入库）→ 一页一 run 对话式追加 → T1 逐值过闸 → 固化进 Git → 落地后真源在代码库。
