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

### Design 基线（参考设计型，2026-09-17 起生效）

本项目 Web 域生效的 design 基线是**参考设计型**：治理规则见 `docs/base_line/基线-design-参考设计-Web域.md`；视觉真源 = 根目录 `DESIGN.md`（改造件，**已断源——禁止对其执行 `designmd.sh update`**）；参考件在 `docs/design-ref/`（只读比对基准）。界面实现的唯一值来源是 `DESIGN.md`，`web/src/app/globals.css` 逐 Token 抄入它（check.sh 内 `designmd validate` 防漂移）。本地化调整清单见 `.scratch/stripe-restyle/spec.md` §2.2；验收 Checklist / 红线 / AI 硬约束见同文档流程节（已定稿，后续实例沿用）。`docs/base_line/基线-design-Web域.md` 是自有风格基线的镜像，本项目**不遵循**它。
