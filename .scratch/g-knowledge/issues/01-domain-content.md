# 01 · 域层与内容层：build.rs 扩展 + knowledge.rs + 16 篇内容 TOML

Status: open
Labels: ready-for-agent

> g-plan T1。规格：arch-g.md §0 ADR-G-001、§2.1/2.2；RULE-038/039/040/044 口径以 prd-g.md §6 为准。

## 步骤

1. `build.rs`：扩展扫描 `config/knowledge/*.toml`（同模式库机制，rerun-if-changed），生成内嵌清单；**不得触碰 ShareType/RuleKind/engine.rs**
2. `domain/knowledge.rs`：`KnowledgeArticle`/`Section` schema + kind 枚举 + `KnowledgeLibrary::load_embedded()`；装载校验全量实现（id 唯一 / sections 非空 / 解读类 related_mode 存在于模式库且必含「局限性」节 / 不可落地类必含「为什么不建议照搬」节 / 其余 kind 禁带 related_mode）
3. 起草 16 篇内容 TOML（**本期主体工作量**；AI 起草、苑问审读为票 07 人工环节）：
   - 解读 ×4（`ki-four-accounts` / `ki-fifty-30-20` / `ki-four-pots` / `ki-snp-quadrant`）：每篇四节——理论来源（含书目 ISBN）/ 核心逻辑 / 适用条件 / **局限性**；与既有 source/credibility 口径一致（标准普尔篇与辟谣文案同源）
   - 考据 ×1（`vf-snp-quadrant`）：流传史 / 证伪要点 / 与 verified 模式的差异
   - 百科 ×8（`ec-rebalancing` / `ec-fees` / `ec-emergency-fund` / `ec-compounding` / `ec-inflation` / `ec-asset-classes` / `ec-dca` / `ec-drawdown`）：每篇 2-3 节，末节固定「与本产品的关系」
   - 不可落地 ×3（`ni-leveraged-lifecycle` / `ni-merrill-clock` / `ni-yale-model`）：含「为什么不建议照搬」节（杠杆不可落地 / 周期只能事后观测 / 机构资源不可复制）
   - **合规红线（RULE-044）**：全文零产品名/代码/链接、零收益承诺、局限性如实写
4. 金例单测：装载 16 篇分组计数 4/1/8/3；解读缺「局限性」→ 失败；related_mode 指向不存在模式 → 失败；不可落地缺「为什么不建议照搬」→ 失败；id 重复 → 失败；未知 kind → 解析期拒绝

## DoD

- `cargo test` 全绿（新增金例全过）；clippy 零警告
- 启动日志含知识库装载计数（16 篇）；故意破坏一条校验 → 启动失败（单测覆盖即可）
- `git diff` 确认 engine.rs 零改动；16 篇 grep 产品名/链接零命中
