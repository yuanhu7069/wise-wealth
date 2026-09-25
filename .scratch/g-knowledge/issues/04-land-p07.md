# 04 · P07 落地：路由 + 四板块 + 阅读态 + T2 验收

Status: resolved
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

## Comments

- 2026-09-25 · AI · 完成。`web/src/app/knowledge/` 五件:page(server 壳,三态 + 深链预取)、knowledge-view(四板块分区,序固定:解读→考据→百科→不可落地)、article-card(客户端岛:展开经 Server Action 取全文,深链预取作 initial)、loading(同族骨架)、state/types。`/knowledge` 进 requireSession 链 + trackPageView p07。
- 2026-09-25 · AI · **执行中的口径决定(已回填)**:① 解读卡徽章由列表响应携带 —— DTO `KnowledgeListItemView` 增 `credibility`(服务端从模式库富化;首版想在前端富化属设计错误,后端富化才是唯一真源),api-types 同步移植;② 文章段落 React key 用内容前缀(biome noArrayIndexKey;内容静态安全);③ 生成文件拆分(knowledge_sources.rs 独立,防 dead_code 警告)—— 已在票 01 落。
- 2026-09-25 · AI · T2 验收:清单外零偏离;走查(生产模式,截图存 shots/):AC-1 四板块 4/4 ✅;AC-2 展开解读含「局限性」不弱化 ✅;AC-12a 深链命中阅读态 ✅ / AC-12b 未知 id 404 文案 ✅;暗色 + 375px ✅。check.sh 5/5 全绿。
