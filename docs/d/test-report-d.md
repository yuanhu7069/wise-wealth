# d 期走查记录(od-redesign)

## 自动门禁

`bash scripts/check.sh` 全绿(clippy / cargo test 94 / biome / tsc / Token 门禁)——每次落地后复跑,最终状态 2026-09-21 ✅(`3974f76`)。

## T1 草稿验收(自动项,AI 执行)

| 页 | token 比对 | 自包含 | 数值/无障碍机制 |
|---|---|---|---|
| P01 | 17/17 | 零外链 | tabular×4、焦点×3、44px×8、对比度 5.25:1(亮)/≈6.6:1(暗) |
| P04 | 11/11 | 零外链 | tabular×4、焦点×3、44px×6、role=status/alert×4、reduced-motion |
| P03 | 10/10 | 自包含(data-uri favicon) | tabular×3、焦点×3、44px×9、aria-pressed×27、progressbar×4 |
| P02 | 10/10 | 自包含(data-uri favicon) | 焦点×3、44px×6、autocomplete×3、aria-describedby、reduced-motion |

## T1 人眼 + T2 走查(苑问)

- 2026-09-20 · P01 即时走查:三形态 / 亮暗 / 375px 通过(逐页流程期)
- 2026-09-21 · 统一复核:四页产物并排(review.html)+ 实机 localhost:3000 全流程(登录 → 问卷 → 方案),结论「页面没问题」——通过
- 走查插曲:dev server 对 next/font 移除 + @theme 全量重写热更失败,重启后验证新编译 CSS 生效(#0969da 3 处 / 旧 #533afd 0 处)

## 已知偏离

全部在 `.scratch/od-redesign/spec.md` §4.1(#1–#8:中文字重/行高/字距、CJK 字体栈、按钮微字距、内距近似、lucide 图标、骨架增强),**无清单外偏离**(T2 判据)。
