# 03 · P06 生成与固化：T0 自查 → 生成 → T1 验收 → 固化

Status: open
Labels: ready-for-agent

> f-plan T3。前置：02 完成（接口就绪便于落地，但生成仅依赖 design-f）。规格：design-f.md 全文；customInstructions = `.scratch/f-modes/custom-instructions.md`（已落盘）。

## 步骤

1. **生成前授权**：向苑问确认本次为真实 BYOK 调用（预算 1 run，≈5-10 min），获授权后才开始
2. T0 自查：四件套（github / redesign-existing-projects / high-fidelity / customInstructions 在库 @ Git）逐项核对；daemon 连通性验证（127.0.0.1:7456）；连不上 → 停下报告
3. 复用 project `ww-redesign-github` **对话式追加**生成 P06（提示词以 5 模式列表为样，含标准普尔存疑卡对照、展开双态、桌面双列 + 375px 单列）；不从零重生成
4. T1·A 自动比对：产物 token 逐值对照 P01 固化契约（基准 = p01-home-github.html + globals.css）；T1·B 自动项（自包含 / 焦点环 / 44px / reduced-motion / 徽章文字+色 / 收起展开双态）；人眼项（亮/暗 × 375px 逐态）列表交苑问按表勾
5. 未过项 → 对话式修改迭代（不批量重跑）；过闸后固化：快照存 `docs/design/f-modes/p06-modes-github.html`（单文件自包含）+ `docs/design/生成记录.md` 追加一行（四件套 / run id / 耗时 / sha256 前 16 位）
6. `globals.css` 原则零改动——产物若需新 token，先进调整清单论证（spec.md §4），默认拒绝发明新值

## DoD

- T1·A 全中 + T1·B 自动项全过；苑问人眼项勾选完成
- 快照文件与生成记录行落盘（Git 内）；sha256 记录一致
- 生成记录行含 run id / 耗时 / 四件套四项，可回答「这套界面怎么来的」
