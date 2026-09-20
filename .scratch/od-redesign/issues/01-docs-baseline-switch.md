# 01 · 文档收尾：基线切换落库

Status: resolved
Labels: ready-for-agent

> 先于一切生成执行（spec §6.2）：不先做，生成流程与仓库现行指令冲突。

## 步骤

1. `AGENTS.md` Design 基线段：改指 `基线-design-OpenDesign-Web域.md`（生成型），删去 DESIGN.md 唯一值来源 / 断源 / designmd validate 等旧叙事；注明 stripe-restyle「后续实例沿用」条款随旧基线作废
2. `DESIGN.md` → 搬入 `docs/design-ref/`（历史参考）
3. `scripts/check.sh`：摘除第 6/6 步 `designmd validate`，后续步骤编号顺延
4. 删 `design-lock.json`
5. 保留 `web/scripts/check-tokens.sh` 及其 check.sh 接线不动

## DoD

- 全仓 `grep -r "designmd" scripts/ AGENTS.md` 无残留引用
- `bash scripts/check.sh` 全绿（其余 5 步不受影响）
- CONTEXT.md「设计治理」词汇已在（grilling 阶段已完成）

## Comments

- 2026-09-20 · AI · 完成。AGENTS.md Design 段改写为生成型叙事(真源=生成动作,三道闸门,stripe-restyle 沿用条款作废);DESIGN.md → `docs/design-ref/DESIGN.stripe-restyle.md`(改名避让未来同名);`design-lock.json` 删除;check.sh 摘第 6 步并顺延为 5/5;`globals.css` 文件头真源叙事同步改写(值出处改为 stripe 期遗留+待固化产物替换)——此项超出原 ticket 步骤,为消除"根目录 DESIGN.md"失效引用所必需,一并报告。组件内 DESIGN.md 字样均为出处标注,留待各页落地时随组件重写。
- 2026-09-20 · AI · 验证:`bash scripts/check.sh` 全绿(clippy/94 测试/biome/tsc/Token 门禁);`grep designmd scripts/ AGENTS.md` 零残留。
