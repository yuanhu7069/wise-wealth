# d 期设计文档 · od-redesign(Open Design 生成型首期)

> 基线:`docs/base_line/基线-design-OpenDesign-Web域.md`(Open Design MCP 生成型 v0.1,2026-09-20 生效)
> prd / arch:**沿用** `docs/智策理财_PRD_V1.1.md` / `docs/b/arch-v2.md` —— 本期功能零变化(界面重设计),无需求与技术栈变更。

## 1 结果

四页界面全部按生成产物重设计并落地:P01 首页 / P02 登录 / P03 问卷 / P04 方案,另含路由级 loading 与 404。视觉体系:GitHub design system(品牌库)。

**落地后真源 = 代码库**:`web/src/app/globals.css`(GitHub 契约值,`@theme`)+ 组件层。生成产物快照与元数据在 `docs/design/`(快照 4 份 + `生成记录.md`),项目运行不依赖 Open Design。

## 2 决策记录(苑问逐项拍板,过程见 `.scratch/od-redesign/spec.md` §2)

| 决策 | 结论 |
|---|---|
| design system | `github`(三候选对比法胜出;Linear / Duolingo 落选) |
| skill / fidelity | `redesign-existing-projects` / high-fidelity |
| 落地策略 | token 名全数保留、值切契约 → 全站即时换肤,结构各页适配 |
| 旧基线处置 | 参考设计型退役,DESIGN.md → `docs/design-ref/DESIGN.stripe-restyle.md` |
| 品牌色 | 不预锁,复核确认无保留需求 |

## 3 闸门记录

| 页 | run(胜者 project `ww-redesign-github`) | T1 自动 | T1 人眼 | T2 |
|---|---|---|---|---|
| P01 | `f1c337c3`(3 候选对比胜出,21.9 min) | ✅ | ✅ 即时走查 | ✅ + 苑问走查 |
| P04 | `3151347e`(对话式追加,9.4 min) | ✅ | 统一复核 ✅ | ✅ 代码级 |
| P03 | `0fa0a162`(对话式追加,9.5 min) | ✅ | 统一复核 ✅ | ✅ 代码级 |
| P02 | `f64bd6af`(对话式追加,4.0 min) | ✅ | 统一复核 ✅ | ✅ 代码级 |

调整清单与派生规则:`.scratch/od-redesign/spec.md` §4(清单 8 项 + 暗色派生规则 + 静态值接动态说明 + 全局刻度决策)。

## 4 工件索引

- 产物快照:`docs/design/od-redesign/`(p01/p02/p03/p04)
- 生成记录:`docs/design/生成记录.md`(四件套 / run / 候选过程 / 派生规则全录)
- 复核页:`.scratch/od-redesign/shots/review.html`
- 过程工单:`.scratch/od-redesign/issues/01-09`
