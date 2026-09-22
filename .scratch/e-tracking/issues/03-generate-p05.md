# 03 · P05 生成与固化：T0 自查 → 生成 → T1 → 固化

Status: open
Labels: ready-for-agent

> e-plan T3。规格：design-e.md 全文；customInstructions：本目录 `custom-instructions.md`（已在库）。
> ⚠️ 开生成前向苑问打招呼获一次授权（spec §5）；daemon 连不上停下报告，不手搓。

## 步骤

1. **T0 自查**：design system `github` / skill `redesign-existing-projects` / fidelity high-fidelity / customInstructions 本目录文件——四项 + 落盘核验，缺一不开跑
2. daemon 连通性验证（127.0.0.1:7456）；生成请求为**对话式追加**到 project `ww-redesign-github`（不新建 project），提示词内联 custom-instructions.md 全文
3. 样例数据按四账户 4 桶给；产物必须含 3 桶/4 桶两布局与全部五态（录入错误态输入保留 / 历史空态 / 无方案空态 / 骨架 / 成功确认 / 偏离条两态）；不满 → 对话式修改迭代，不整页重生成
4. **T1·A**：token 逐值比对（基准 = `docs/design/od-redesign/p01-home-github.html` 17 项契约）；**T1·B**：自动项跑（自包含/对比度脚本/tabular/焦点/44px/reduced-motion），人眼项列清单交苑问勾（亮/暗 × 375px 逐态）
5. 未过项迭代至过；连续 2 轮漂移 → 停，交苑问裁决（RISK-E-2）
6. **固化**：产物副本 → `docs/design/e-tracking/p05-tracking-github.html`；`docs/design/生成记录.md` 补一行（四件套/run id/耗时/sha256 前 16 位）；`globals.css` 原则零改动（确需新 token → 先进调整清单论证，默认拒绝）

## DoD

- T1·A 全中；T1·B 自动项 ✅、人眼项苑问已勾
- 快照文件 + 生成记录行落盘（Git 内可溯源）
- 红跌绿涨语义核对（相反则纠正并记调整清单草案）

## Comments

-
