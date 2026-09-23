# 03 · P05 生成与固化：T0 自查 → 生成 → T1 → 固化

Status: resolved
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

- 2026-09-23 · AI · 苑问授权后一次生成(run `0c16a225`,7.7 min,对话式续接四页会话,迭代轮次 0)。T1·A:绑定契约 56 项逐值全中,全文档 hex 超出冻结契约 **0 个**(无任何新值);T1·B 自动项全过(自包含/tabular×5/焦点环/44px×10/role×5/reduced-motion/aria-busy)。红涨绿涨语义核对:delta-up=success 绿、delta-down=danger 红,与 customInstructions 既定规则一致;负数红字 ✓。固化:快照 `docs/design/e-tracking/p05-tracking-github.html`(sha256 前 16 位 `a8b1142185d252f5`)+ 生成记录补行,已提交 `335b3c8`。
- 2026-09-23 · AI · **待苑问**:① T1 人眼项(亮/暗 × 375px 逐态)按 design-e §4 清单勾选;② 产物把偏离上涨文案定调为「多花了 X%」(增长绿)——语义上「多花钱」用绿是否合意,人眼走查时一并定;③ 主题切换为三态循环(与前四页行为一致,非偏离)。
- 2026-09-23 · AI · T1 人眼项未勾前不落地(红线 9)。两个走法待苑问选:① 现在走查(亮/暗 × 375px 逐态,清单见 design-e §4);② 沿 D 期「批量授权」先例——落地后期末统一复核(该节奏是 D 期专批,本期沿用需苑问再次确认)。
