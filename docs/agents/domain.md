# Domain Docs

How the engineering skills should consume this repo's domain documentation when exploring the codebase.

## Before exploring, read these

- **`CONTEXT.md`** at the repo root (this repo is single-context; no `CONTEXT-MAP.md`)
- **`docs/adr/`**: read ADRs that touch the area you're about to work in

If any of these files don't exist, **proceed silently**. Don't flag their absence; don't suggest creating them upfront. The `/domain-modeling` skill (reached via `/grill-with-docs` and `/improve-codebase-architecture`) creates them lazily when terms or decisions actually get resolved.

## 本仓库补充：与既有文档体系的关系

- 本仓库另有**期级工程文档**（`docs/<期号>/`：prd/arch/design/plan/test-report）与**产品 PRD**（`docs/智策理财_PRD_V1.1.md`）。探索领域概念时，先读 `CONTEXT.md`（如已存在），再读产品 PRD 的相关章节；工程决策（ADR 形态）目前落在各期 arch 文档的 ADR 小节（如 arch-a.md ADR-A-001~004）。
- `docs/adr/` 保留给 `/domain-modeling` 今后产出的领域决策；两者编号空间独立，互不改写。

## File structure

Single-context repo:

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/                  ← 领域 ADR（/domain-modeling 惰性创建）
│   └── <期号>/               ← 期级工程文档（本仓库既有约定）
└── src/
```

## Use the glossary's vocabulary

When your output names a domain concept (in an issue title, a refactor proposal, a hypothesis, a test name), use the term as defined in `CONTEXT.md`. Don't drift to synonyms the glossary explicitly avoids.

If the concept you need isn't in the glossary yet, that's a signal: either you're inventing language the project doesn't use (reconsider) or there's a real gap (note it for `/domain-modeling`).

## Flag ADR conflicts

If your output contradicts an existing ADR (in `docs/adr/` or in a phase arch doc), surface it explicitly rather than silently overriding:

> _Contradicts ADR-A-003 (health 恒 200), but worth reopening because…_
