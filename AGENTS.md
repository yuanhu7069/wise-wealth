# wise-wealth · Agent 指令

> 本文件是仓库的 agent 指令正文；CLAUDE.md 通过 `@AGENTS.md` 引入本文件，内容单点维护，不要在两处重复编辑。
> 项目背景与开发方法见 `docs/a/a-claude-communication-records.md`；各期实例文档在 `docs/<期号>/`。

## Agent skills

### Issue tracker

Issues 以本地 markdown 文件跟踪，存放于 `.scratch/<feature-slug>/`（一 feature 一目录：spec.md + issues/NN-<slug>.md）。See `docs/agents/issue-tracker.md`.

### Triage labels

使用五个默认角色标签，标签字符串即角色名：`needs-triage` / `needs-info` / `ready-for-agent` / `ready-for-human` / `wontfix`。See `docs/agents/triage-labels.md`.

### Domain docs

Single-context：根 `CONTEXT.md`（由 /domain-modeling 惰性创建，缺失时静默跳过）+ `docs/adr/`（领域 ADR；工程期 ADR 在各期 arch 文档内，两者并行）。See `docs/agents/domain.md`.

### Design 基线（Open Design MCP 生成型，2026-09-20 起生效）

本项目 Web 域生效的 design 基线是**Open Design MCP 生成型**：治理规则见 `docs/base_line/基线-design-OpenDesign-Web域.md`。设计真源是生成动作：生成前固定参数四件套（design system / skillId / fidelity / customInstructions）→ 草稿过 T1 验收 → 固化进 Git（tokens 进 `web/src/app/globals.css` 的 `@theme`、产物快照与生成记录进 `docs/design/`）→ 落地后真源在代码库。旧参考设计型基线已退役：原根目录 `DESIGN.md` 移入 `docs/design-ref/DESIGN.stripe-restyle.md` 仅作历史参考；stripe-restyle 期「验收 Checklist / 红线 / AI 硬约束后续实例沿用」条款随之作废，本期起验收以生成型基线三道闸门（T0/T1/T2）为准；`check-tokens.sh` 门禁保留（代码卫生，与真源无关）。
