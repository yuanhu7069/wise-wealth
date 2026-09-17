# 01: 治理与参考件落地（基线拷入 + AGENTS 指针 + 锁源）

**What to build:** 把参考设计型治理基线的落地件放齐：仓库内基线副本、AI 入口指针、参考件与锁源入库确认。本 ticket 只动治理与文档，不写一行样式代码。

**Blocked by:** None（可立即开工）

**Status:** done（2026-09-17）

**完成记录（2026-09-17）**：五项全部完成。基线已镜像至 `docs/base_line/`（12185 字节与外置主档一致）；参考件锁源 `commitSha 8147538b` 与 spec §2.1 一致；`git check-ignore` 确认 `design-lock.json`、`docs/design-ref/`、`.scratch/stripe-restyle/` 均入库；AGENTS.md 新增「Design 基线（参考设计型）」一节指针。gate：validate 于拉取时已 PASS（02 中挂入 check.sh 后为常设门禁）。

- [x] 拷贝 `~/project-baseline-docs/基线-design-参考设计-Web域.md` → `docs/base_line/基线-design-参考设计-Web域.md`（镜像外置主档；旧基线 `基线-design-Web域.md` 保留不动）
- [x] 参考件 `docs/design-ref/DESIGN.stripe.md` 已由 `npx designmd.sh add --output` 拉取（2026-09-17，源锚点 `main@8147538b`），保持原样只读
- [x] `design-lock.json`（仓库根）内容核对：来源路径与 commitSha 与 §2.1 一致，确认未被 .gitignore 忽略
- [x] 根 `AGENTS.md` 增加一节指针：本项目 Web 域生效 design 基线 = 参考设计型；视觉真源 = 根目录 `DESIGN.md`（改造件、已断源、禁止 update）；旧基线为镜像不遵循
- [x] 确认 `docs/design-ref/` 与 `design-lock.json`、`.scratch/stripe-restyle/` 均入库

**切分理由：** 治理件是后续所有 ticket 的依据（02 起每一步都要引用基线与参考件），必须最先落位；且全部是文件操作零代码风险，单独成 ticket 便于审。
