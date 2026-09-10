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
