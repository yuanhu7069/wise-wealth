# 03 · P09 生成与固化：T0 自查 → 生成 → T1 验收 → 固化

Status: ready-for-agent
Labels: ready-for-agent

> h-plan T3。规格：design-h.md 全文（T0 四件套 / 生成预算 / T1 判据 / 固化要求）。
> **生成前需苑问授权一次**（BYOK 真实调用，红线：不批量重跑）。

## 步骤

1. T0 自查：四件套齐备——design system `github`（digest `5b03d62f…`）/ skill `redesign-existing-projects` / fidelity high-fidelity / customInstructions `.scratch/h-plus/custom-instructions.md`（已在库）；daemon 只做连通性验证，连不上 → 停下报告，不手搓
2. 生成：复用 project `ww-redesign-github` 对话式追加（不新建 project）；提示词样例按 design-h §3（三卡 + 切换器 + 空态 + 单点形态 + 特殊月标注 + 免责页脚）
3. T1·A：产物 token 值逐项对照 P01 固化契约（`docs/design/od-redesign/p01-home-github.html` + `globals.css`），对不上即漂移判失败
4. T1·B 六项：五态（骨架/错误/空季/正常/单点）、极端数据不破版、中文渲染、自包含、无障碍、数值展示（带符号 + 负数红字 + 无完成度百分比）；自动项 AI 跑，人眼项（亮/暗 × 375px 逐态）交苑问
5. 固化（生成后立即，P0）：产物快照 → `docs/design/h-plus/p09-review-github.html`（单文件自包含）；`docs/design/生成记录.md` 追加一行（四件套 / run id / 耗时 / sha256 前 16 位）；`globals.css` 原则零改动（确需新 token → 先进 spec §4 调整清单论证，默认拒绝）

## DoD

- T1·A 全中 + T1·B 自动项全过；人眼项清单交苑问（票 04 落地后统一复核）
- 快照 + 生成记录落盘；产物 sha256 记录在案
- **苑问授权记录**写入本票 Comments（授权时间 + run id）

## Comments

-
