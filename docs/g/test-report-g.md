# 测试报告 · 智策理财 G 期(知识库 V1 与模式对比)

> 对应需求:REQ-20260925-02(prd-g.md)· 执行期:2026-09-25(单日,票 01-07)
> 验收口径:prd-g 元信息块 DoD 六条;红线来源:基线-arch §18(19 条)
> 本报告状态:**技术验收全绿**;苑问内容审读(16 篇)、人工走查与 DoD 验收动作待执行(见 §9)

---

## 1. 自动门禁与红线

`bash scripts/check.sh` **5/5 全绿**(收口时点复跑):

| # | 门禁 | 结果 |
|---|------|------|
| 1 | cargo fmt + clippy(--all-targets) | ✅ 零警告 |
| 2 | cargo test(156 用例) | ✅ 156 passed / 0 failed |
| 3 | biome check(src + scripts) | ✅ |
| 4 | tsc --noEmit | ✅ |
| 5 | Token 门禁(check-tokens.sh) | ✅ |

**期级声明的 diff 级核对**:
- **零数据库迁移**(连续第三期):无新迁移文件;知识内容为构建期内嵌(ADR-G-001)✅
- **零引擎改动**:`engine.rs`/`ShareType`/`RuleKind`/求解序未触碰;preview 只读复用 `solve` ✅

## 2. 测试构成

**域/加载层金例单测**(新增 12 用例):

| 用例 | 断言 |
|------|------|
| 内嵌知识装载 | 16 篇齐,kind 分组计数 4/1/8/3,id 全库唯一 |
| 解读校验 | related_mode 必带且必须存在于模式库;必含「局限性」节(RULE-039) |
| 不可落地校验 | 必含「为什么不建议照搬」节;禁带 related_mode(RULE-040) |
| 内容合规机械项 | 段落含 http/https 链接 → 启动失败(RULE-044);空段落/空标题拒绝 |
| id 重复 / 未知 kind | 分别在装载/解析期拒绝(fail-fast) |
| share_desc | 四种 share 类型字符串化正确 |
| **preview 对账** | 同档案同模式下 preview.solution 与生成路径 solve **逐字段一致**(四模式全对账,AC-6 自动化底座) |
| preview 条目元数据 | 含 credibility/source/buckets_meta(share_desc 非空) |
| entry / preview 边界 | mode_ids 空/超 3/未知 id → 422 |

**service/API 集成**:3 新端点无 Cookie → 401;未知文章 id → 404;`ni-*` 条目进 preview → 422(内容与引擎严格分离,RULE-040)。

## 3. 设计闸门(T0 → T1 → T2 ×2 页)

| 闸门 | P07 知识库 | P08 对比页 |
|------|-----------|-----------|
| T0 四件套 | ✅ github / redesign-existing-projects / high-fidelity / customInstructions @ 8f9612b | 同左 |
| 生成 | run `d21612a8`,≈6.5 min,**轮次 0** | run `5faeae84`,≈5.5 min,**轮次 0** |
| T1·A 一致性 | ✅ 契约 63 项逐值全中 + hex 超出 0 | ✅ 契约 63 项逐值全中 + hex 超出 0 |
| T1·B 完整性 | ✅ 自动项 17/17(四板块/徽章三色/局限性不弱化/警示徽章/五态/极端核验) | ✅ 自动项 13/13(三列样本/金额等宽右对齐/不可行警示/零勾选引导/横滑) |
| T2 落地 | ✅ 调整清单 4 项(spec §4),清单外零偏离 | ✅ 清单外零偏离 |

固化齐备:快照 `docs/design/g-knowledge/p07-knowledge-github.html`(sha256 前 16 位 `e7c5d6b2ce232113`)、`p08-compare-github.html`(`1e733fefe4672786`)+ 生成记录两行 + globals.css 零改动。

## 4. 实机走查(生产模式;截图存 `.scratch/g-knowledge/shots/`)

> 环境备注延续 F 期:turbopack dev 在 headless 环境下 hydration 不发生(非本期代码回归);走查在生产模式完成,前端已恢复 dev 驻留。

| AC | 场景 | 结果 | 证据 |
|----|------|------|------|
| AC-1 | P07 四板块 + 埋点 | ✅ | p07-desktop-light.png;埋点 §5 |
| AC-2 | 解读四节 + 局限性不弱化 + 展开全文 | ✅ | p07-article-expanded.png |
| AC-3 | 考据全文 + 口径一致(从未发布) | ✅ | 深链阅读态 + TOML 与徽章同源 |
| AC-4 | 不可落地标注 + 引擎分离 | ✅ | 警示徽章(截图);preview 对 `ni-*` → 422 实机 |
| AC-5 | 勾选 → 对比表格(URL 态) | ✅ | p08-compare-table.png |
| AC-6 | 试算对账一致 | ✅ | 单测四模式对账 + 实机(四笔钱 6,000/0/4,000/10,000 与 v12 真实生成一致) |
| AC-7 | 存疑列徽章 + 金额照常试算 | ✅ | p08-compare-table.png |
| AC-8 | 取数失败错误态 | ✅ | 组件沿 F 期同构(实机路径:后端不可达统一落登录重定向,同 F 期 AC-11 说明) |
| AC-9 | 勾选超限提示 | ✅ | p06-compare-select.png(实机第 4 个被拒) |
| AC-10 | 不可行列警示短语 | ⚠️ 验证缺口 | 引擎金例(insufficient_income)+ 前端分支代码审读;当前档案收入充足无法实机触发 |
| AC-11 | 375px 横滑首列固定 | ✅ | p08-mobile-375.png(实机 scrollWidth 验证) |
| AC-12 | 深链命中 + 未知 id 404 | ✅ | p07 深链阅读态 + 404 文案实机 |
| AC-13 | 会话过期回跳 | ✅ | 清 cookie → login?from=/knowledge 实机 |
| AC-14 | 返回勾选保持 + 解读跳转 | ✅ | 返回 3/3 保持;解读跳转实测(id 对齐修正后) |

五态矩阵对照 prd-g §8.3 全部落地 ✅。

## 5. 埋点 SQL 查验

```
page_view  page_id=p07  ≥3 条;page_id=p08  ≥2 条(2026-09-25 走查期)
金额入载荷              0 条(正则复核 cents|余额)
```

## 6. 备份与恢复演练

`restore-drill.sh` 复跑(2026-09-25):12 项抽验全一致;副本库起服务 `/health db=ok` + 种子登录 + 读方案数据全通。零迁移故无新增抽验项。

## 7. 安全审计

| 项 | 结果 | 处置 |
|----|------|------|
| cargo audit | RUSTSEC-2023-0071(rsa,medium,无修可升) | 既有挂账,非本期引入 |
| npm audit | js-yaml high ×2(存量传递依赖) | 挂账;是否 fix 留苑问拍板 |
| 内容合规 | 16 篇 grep 链接零命中;产品名零命中;loader 机械拦截链接(RULE-044) | 内容审读待苑问 |
| 输入面 | mode_ids 数量/存在性、文章 id、白名单零变更 ✅ | 集成测试 |

## 8. 已知偏离与遗留

**偏离(spec §4,待苑问拍板)**:文章阅读态用 Server Action 按需取全文(产物为内嵌示意);深链以查询参数承载(非子路由)。

**遗留**:
1. gen-types.sh TS7 修法(ADR-G-004 升格拍板项;本期手工移植 Knowledge*/Preview* 共 10 schema + 2 次增补)
2. AC-10 实机验证缺口(待首个低收入档案或 mock)
3. F 期挂账延续:金字塔(OUT-003)/ TLS / dev hydration / caution 徽章缺口 / I-8/9/12
4. `web/src/app/knowledge/state.ts` 与 `modes/state.ts` 各有一份 CREDIBILITY_BADGE(轻度重复,Phase 2 抽公共)

## 9. 评审与修复记录 + DoD 判定

**执行中修复**(全部当天闭环):
- 票 02→04:列表 DTO 富化 credibility(徽章唯一真源在后端,前端富化属设计错误已纠正)
- 票 04:api-types 手工移植口径修正(components 导入);biome key/格式化
- 票 06:解读文章 id 连字符→下划线对齐模式 id(走查暴露即修);生成文件拆分防 dead_code(票 01)

**DoD 判定**(prd-g 元信息块):

| 项 | 状态 |
|----|------|
| 功能:AC-1~14 | ✅ 技术全过(AC-10 缺口见 §4) |
| 内容:16 篇可数达标 | ✅ 4+1+8+3 装载锁定;**苑问内容审读待执行** |
| 体验:全链路 亮/暗+375px | ✅(截图;**苑问人工走查待执行**) |
| 质量:test-report-g + 红线 + check.sh | ✅ 本报告;红线 19 条逐项 ✅(红线 1 零新依赖 / 红线 2 RULE 注释落位 / 红线 15 零迁移 / 其余沿既证) |
| 设计:两页三道闸门 + P06 回归 | ✅ §3;P06 既有功能回归无破坏 |
| 运维:零迁移 + restore-drill + 一键启动 | ✅ §6 |
| 我的验收动作(苑问) | **待执行**:实机走对比全链路(勾 3 → 试算表格 → 深读考据)+ 16 篇内容审读 |

**结论**:技术验收全绿。人工验收:2026-09-26 苑问完成 16 篇内容审读与页面走查(校验通过,暂验收口径——后续有问题再开票),DoD 验收动作随走查覆盖。prd-g →「已完成」,期收口。
