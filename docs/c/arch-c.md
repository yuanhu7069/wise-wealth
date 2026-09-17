# C 期 · Stripe 参考设计风格切换（Web 域视觉基线换代）

> 2026-09-17 交付。本文是期级实例记录；过程文档（spec / issues / 走查截图）在 `.scratch/stripe-restyle/`。

## 摘要

按参考设计型治理基线（`docs/base_line/基线-design-参考设计-Web域.md`）完成首次落地：以 Stripe 为参考，对 Web 前端做标准重排——token 全量换代 + 组件形态重排 + 四页适配 + 首页渐变 mesh hero。亮/暗两态、四页、移动端实机走查通过，check.sh 六步门禁全绿。

## 基线换代事实（本文档最重要的记录）

- **本项目 Web 域生效的 design 基线自 2026-09-17 起为参考设计型**：治理规则 `docs/base_line/基线-design-参考设计-Web域.md`；**视觉真源 = 仓库根 `DESIGN.md`（改造件，已断源，禁止 `designmd.sh update`）**。
- 自有风格基线 `基线-design-Web域.md` 本项目不再遵循（文件保留：外置主档镜像 + docs/a、docs/b 历史引用）；松墨绿 `#4A6B57` 色板随本次切换退役。
- 防漂移机制：`designmd validate` 作为 `scripts/check.sh` 第 6 步常设门禁；`design-lock.json` 锁源（`voltagent/awesome-design-md` @ `8147538b`）；参考件只读存于 `docs/design-ref/`。
- 后续新增功能的界面一律以根 `DESIGN.md` 为唯一值来源；偏离逐条记入实例文档调整清单（模板见 `.scratch/stripe-restyle/spec.md` §2.2）。

## 本地化调整（相对参考件的全部偏离，完整理由见 spec §2.2）

Inter 替代 Sohne（中文字重 400、display 零负字距）；micro-cap 11px；本地语义色（success/warning/danger/danger-strong，实测定值）；红涨绿跌；图表固定六色（值全取参考色板）；暗色模式整体派生（深靛调）；mesh 仅首页；触控 ≥44px；canvas 暗色拆档；辅助文字在 canvas-soft 底用 ink-mute-2。

## 走查与质量

- 门禁：clippy / cargo test / biome / tsc / Token 门禁 / designmd validate 全绿；`next build` 通过。
- 实机走查：种子账号完整核心路径（登录 → 问卷续答与六步作答 → 生成方案 → 方案页），亮/暗 × 四页 + 移动端 16 张截图，存 `.scratch/stripe-restyle/shots/`。
- AI 视觉评审两轮：首轮 4 项 fail（mesh 浑浊硬接缝、暗色选中卡对比崩坏、一张截图错位）→ 修复 → 复验 9/9 pass。
- 对比度实测：亮/暗 34 组实际配对全部达标（脚本 `.scratch/stripe-restyle/tmp-contrast-check.py`）；实测逼出 6 处调深（down/up/success/warning/danger-strong），全部回填 DESIGN.md 与 spec。

## 遗留

- spec 流程节（验收 Checklist / 红线 / AI 硬约束）为 AI 起草稿，**待苑问改定**（治理基线 §0.3 归苑问给出）。
- 定价/对比类页面出现时再启用 DESIGN.md 中预留的 card-pricing 组件定义。
- 前端仍无自动化测试；截图回归（Playwright）留待后续期评估。
- mesh 暖色档（lemon token）观感偏沙褐：如后续觉得偏土，可降透明度或去暖斑（不发明新色值），2026-09-17 评审通过时维持现状。

## 终验与追加修正（2026-09-17）

苑问人工校验走查**通过**，验收成立。走查后按其反馈追加两项修正（均经实测确认）：
1. 问卷操作栏恒钉视口底——页面外壳全高拉伸 + 操作栏 `mt-auto`，矮/高步骤按钮不再跳动（design-v2 v0.5 意图的真正落地）；
2. 「重新生成」入口（P01/P04）带 `restart=1` 落问卷步 1（答案仍预填，且不重复计「问卷开始」埋点）；步 6 模式装载后默认选中推荐项，「生成方案」不再因未点卡片而禁用。
