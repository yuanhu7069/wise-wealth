# 04 · P07 落地：路由 + 四板块 + 阅读态 + T2 验收

Status: open
Labels: ready-for-agent

> g-plan T4。前置：03 完成（固化产物为落地真源）。规格：arch-g.md §1、§3（RULE-038/039/040）；prd-g.md §8。

## 步骤

1. 新建 `web/src/app/knowledge/`：page（server 壳，取列表）+ 视图（四板块分区：解读卡 / 考据卡 / 词条小卡网格 / 不可落地卡带警示徽章）+ 阅读态（`?id=` 查询参数展开全文 sections；未知 id → 404 文案）
2. `/knowledge` 进 requireSession 重定向链；`trackPageView("p07")`
3. 五态接线：路由级 loading 骨架 / 错误条重试 / 板块空防御（理论不可达）/ 正常
4. 调整清单落 spec.md §4；D/E/F 期结论照抄项不再论证
5. T2 验收：清单外零偏离；走查 AC-1/2/3/4/12（桌面 + 375px × 亮/暗，截图存 shots/）

## DoD

- `check.sh` 全绿（含 Token 门禁）
- 五态对照 prd-g §8.3；AC-1/2/3/4/12 走查过（截图为证）
- 「局限性」「仅供理解」视觉不弱化核对；grep 产品名零命中
