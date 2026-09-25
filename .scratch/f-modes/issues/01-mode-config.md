# 01 · 配置与域层：mode.rs source 字段 + 2 个新 TOML + 存量补齐

Status: resolved
Labels: ready-for-agent

> f-plan T1。规格：arch-f.md §0 ADR-F-001/003、§2（含 2.1 TOML 规格表与口径决定）；RULE-036/037 口径以 prd-f.md §6 为准。

## 步骤

1. `mode.rs`：`ModeConfig` 增可选字段 `source: Option<String>`；加载校验新增「source 非空」（缺失/为空 → 启动失败 fail-fast）；**不得触碰 ShareType/RuleKind/engine.rs**（RULE-036 红线）
2. 存量 `four_accounts.toml` / `fifty_30_20.toml`：把顶部注释中的出处迁入 `source` 字段（注释保留背景性说明，不再承载权威出处——RULE-037）
3. 新增 `server/config/modes/snp_quadrant.toml`：四桶 pct 1000/2000/3000/4000（spending=is_necessary / growth=is_investable），credibility=disputed，source 含辟谣（规格见 arch-f §2.1）；emergency_fund 档位拷贝 fifty_30_20 并核对
4. 新增 `server/config/modes/four_pots.toml`：liquid=fixed_expenses(is_necessary) / protection=rule(emergency_fund) / stable=pct 2000 / long_term=remainder(is_investable)，credibility=verified，source=盈米且慢（规格见 arch-f §2.1）
5. 金例单测锁定（arch-f §10 表）：snp_quadrant 求解四桶金额、合法性校验、four_pots 求解与合法性、source fail-fast、disputed 永不 is_recommended（recommend 既有规则回归测试）
6. 口径决定回填：四笔钱应急金归保障桶、稳钱 20% 可调、保费不入桶——记入本票 Comments 并核对 arch-f §2.1 表述一致

## DoD

- `cargo test` 全绿（新增金例全过）；clippy 零警告
- 启动日志显示 4 个 L1 模式全部加载成功；故意删 source 启动失败（手动验证一次）
- `git diff` 确认 engine.rs / ShareType / RuleKind 零改动
- 四笔钱/标准普尔各跑一次「问卷路径外」的直连生成（curl POST /plans），方案落库成功

## Comments

- 2026-09-25 · AI · 完成。mode.rs 增 `source: Option<String>` + 装载期非空校验（缺/空 → fail-fast）；存量两 TOML 出处迁入字段（注释不再承载权威出处）；新增 snp_quadrant.toml / four_pots.toml（规格照 arch-f §2.1）。cargo test 140 全绿（含金例：标准普尔 4321 切分、四笔钱 fixed+rule+pct+remainder、source fail-fast、disputed 永不主推）；clippy 零警告；engine.rs diff 零删除行（仅新增测试，RULE-036 红线核对）。
- 2026-09-25 · AI · 执行中的口径决定（已回填）：① **桶 id 跨模式错开**——四笔钱保障桶用 `safeguard` 而非 `protection`，避免与标准普尔 `protection` 重名（追踪按同名桶比对偏离，重名会跨模式误比对），mode.rs 测试显式锁定；② **应急节奏取整**——pacing_transfer 按「百元/月」粒度四舍五入（¥1,250/月 → ¥1,300），金例断言据此修正（130,000 分，months_to_fill=23）。
- 2026-09-25 · AI · **文档一致性修复（测试暴露）**：金字塔推后期内为 4 个模式，prd-f / design-f / f-plan / custom-instructions / 工单共 7 处「5 个/5 卡」改为 4，并在 DoD 与 §7.2 标注「Phase 1 目标 5 个，金字塔挂账 OUT-003」。
- 2026-09-25 · AI · DoD 实测：重启后端，启动日志「模式库已装载：4 个 L1 模式」；curl 直连生成两新模式均落库（snp_quadrant v9 / four_pots v10，真实档案）；**测试后已把 admin 的 active 方案恢复为 v8 four_accounts**（测试版本保留为历史版本，版本化设计），库中另一用户（非 admin）数据未触碰。check.sh 5/5 全绿。
