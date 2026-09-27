# 测试报告 · 智策理财 H 期(Plus 起步——推理链展开与季度复盘)

> 对应需求:REQ-20260927-01(prd-h.md)· 执行期:2026-09-27(票 01-04 已收口,票 05 进行中)
> 验收口径:prd-h 元信息块 DoD 七条;红线来源:基线-arch §18(19 条)
> 本报告状态:**技术验收全绿**;苑问人眼项(亮/暗 × 375px 逐态)、T2 调整清单拍板、DoD 验收动作(录 10 月快照看 Q4 复盘)待执行(见 §8)

---

## 1. 自动门禁与红线

`bash scripts/check.sh` **5/5 全绿**(收口时点复跑):

| # | 门禁 | 结果 |
|---|------|------|
| 1 | cargo clippy(--all-targets,-D warnings) | ✅ 零警告 |
| 2 | cargo test(169 用例) | ✅ 169 passed / 0 failed |
| 3 | biome check(src + scripts) | ✅(1 条 warning 为 G 期 knowledge 页遗留,非本期文件) |
| 4 | tsc --noEmit | ✅ |
| 5 | Token 门禁(check-tokens.sh) | ✅(本期被拦两次均已修:trace-panel 裸 px → token 类;注释裸单位) |

**期级声明的 diff 级核对**:
- **零数据库迁移**(连续第三期):迁移止于 0007;`.env.example` 零新增 ✅
- **引擎改动 = 仅让位 trace 追加**(ADR-H-002):`git diff` 审计确认 solve 金额路径/签名/既有输出零变;diff 全文已在票 01 复核 ✅
- **闸门在服务侧**(ADR-H-001):`PlanView.traces` 仅 plus 序列化,free 响应无键(单测 + curl 双证)✅

## 2. 测试构成(新增 15 用例)

**引擎推理链金例**:

| 用例 | 断言 |
|------|------|
| 让位发生时推理链含第四节点 | traces 长度 4;`safety_first_yield` output=0(投资桶归零)、rationale 含让位规则名 |
| 让位未发生时恒为三节点 | 金例 A(资源充足)与金例 B(达标)均无第四节点、不占位 |
| **既有金例全量回归** | solve 金额断言零改动、全绿(154 例);preview 对账单测绿(traces 同源) |

**分层闸门金例**:

| 用例 | 断言 |
|------|------|
| tier 解析 | free/plus 正确;未知库值(enterprise/空串)降级 Free(给得少方向) |
| 闸门双向(to_view 纯函数) | 同一方案:Free 序列化无 `traces` 键 + `trace_count` 恒下发;Plus 全链下发 |
| tier 实时 | service 每请求现读(`users::tier_of`),不进 JWT 不缓存 |

**季度复盘聚合金例**(RULE-049/050,arch-h §10):

| 用例 | 断言 |
|------|------|
| 正常季度三卡数值 | prd-h §7.1 样例逐字:31,000→42,200(+11,200);缺口 42,800→31,600;月均 5,600(÷2 步);约 6 个月攒满 |
| 特殊月口径 | 计入快照数与坚持、**不入环比**(端点跳到 11 月);special_months 列表齐 |
| 全特殊 / 单点 / 空季 | 三者均不可环比且可区分(count 0/1/≥2);单点不给均值不外推 |
| 缺口扩大 | 月均如实负值,months_to_goal=None(倒退不给「还差几个月」) |
| 季内达标 | met=true、缺口 0、无外推 |
| 同名桶可比 | 季内换桶结构后只列两端同有的桶(RULE-028 立场) |
| 观察桶缺席 | 不编应急结论(emergency=None) |
| review 边界 | `2026-Q5`/`abc` → 422;空季 → 200 空集;无 Cookie → 401 |

## 3. 设计闸门(T0 → T1 → T2 × P09)

| 闸门 | P09 季度复盘页 |
|------|----------------|
| T0 四件套 | ✅ github(digest `5b03d62f…`,同前八页)/ redesign-existing-projects / high-fidelity / customInstructions(`.scratch/h-plus/custom-instructions.md`,D 定稿 + H 追加节,已入库) |
| 生成 | run `cb16cde6-e8e7-4480-bbbe-c179069be356`,**5.5 min,轮次 0**;stable-prompt 缓存命中 88.7%;苑问 AskUserQuestion 明确授权(2026-09-27) |
| T1·A 一致性 | ✅ 全文档 hex 超出 P01 冻结契约 **0 个**;color-mix 派生与契约一致 |
| T1·B 完整性 | ✅ 自动项 **20/20**(五态齐备/极端数据核验区/自包含零外链/44px×3/reduced-motion/aria-live/负数红字/全页无完成度百分比/特殊月 pill/外推「不构成承诺」标注/免责声明/中文回退栈) |
| T2 落地 | ✅ 调整清单 **7 项**回填 spec §4(状态单态化/核验区不落地/特殊月标注位置/复用 shell/切换器参数化/标签底色/路由链接)——**待苑问拍板**;清单外零偏离 |

固化齐备:快照 `docs/design/h-plus/p09-review-github.html`(778 行,sha256 前 16 位 `755681fc988af29b`)+ 生成记录补行 + globals.css 零改动。

## 4. 闸门双向断言(RISK-H-1 的实测项)

真实 dev 库全链路(票 02):

| 步骤 | 断言 | 结果 |
|------|------|------|
| free(默认)curl 方案接口 | 响应体无 `traces` 键,`trace_count=3` | ✅ |
| `set-tier.sh admin plus` 后同接口 | `traces` 全量下发(rationale 与方案冻结一致,含赡养上浮文案) | ✅ |
| `set-tier.sh admin free` 还原后 | `traces` 键再次消失 | ✅ |
| **现场还原** | tier 已还原 free | ✅ |

## 5. 实机走查(全栈冒烟,票 04)

> 环境备注:新后端(8080)+ Next dev(3001);shell 的 `http_proxy` 会拦 localhost(曾致假 502),`--noproxy` 复验为准。亮/暗 × 375px 逐态人眼项交苑问(§8)。

| AC | 场景 | 结果 | 证据 |
|----|------|------|------|
| AC-4 | 闸门双向 + 实时生效 | ✅ | §4 curl 全链 |
| AC-7 | P09 进入 + 当前季 + 埋点 | ✅ | 冒烟 200;默认渲染「2026 年第 3 季度」;埋点 SQL 见 §6 |
| AC-10 | 单点形态 | ✅ | 默认 Q3(1 条快照)渲染「本季仅 1 条快照,变化无从比较」;空季文案在页(产物态),直链无数据季同源 |
| AC-11 | 非法 quarter | ✅ | `?quarter=abc` → 200 + 「这个季度格式不对」+ 回当前季按钮,不白屏 |
| AC-1/3 | 「为什么」入口双态 | ✅(服务侧) | P04 页面渲染入口(HTTP 200 + 文案在页);plus 全链/占位双态见 §4 数据;**视觉走查待苑问** |
| AC-2 | 让位节点如实 | ✅(单测) | 让位金例 4 节点/常规恒 3;实机未构造收入不足档案(金例覆盖) |
| AC-5/13 | 过期/取数失败形态 | ✅(沿先例) | 与 F/G 同款内联错误 + 重试,代码路径在页 |
| AC-6/8/9/12/14 | 一致性/三卡/特殊月/距离感/免费全量 | ✅(代码 + 金例) | review 端点不读 tier(实现审查);金例锁定;**375px/亮暗人眼待苑问** |

P05 既有五件事(录入/进度/偏离/历史/导出)与 P04 既有五段:回归零破坏(走查 + 门禁)。

## 6. 埋点 SQL 抽验

```sql
SELECT event_type, payload_json->>'page_id', occurred_at
FROM analytics_events WHERE payload_json->>'page_id'='p09';
-- page_view | p09 | 2026-09-27 09:57:58+00 (×2,冒烟触发)
```
金额与推理链内容零命中埋点与日志(RULE-052;tracing 只带 request_id + 路由名)。

## 7. 备份恢复

`restore-drill.sh` 全绿(2026-09-27):备份 `backups/wise_wealth_20260927_1800.dump`(sha256 `952eea84…`),副本库恢复 + 起服务 `/health db=ok`,抽验 12 项全一致。

## 8. 待苑问事项(期收口前置)

1. **人眼项**:P09 + P04 推理链区,亮/暗 × 375px 逐态走查(产物快照:`docs/design/h-plus/p09-review-github.html`)
2. **T2 调整清单拍板**:spec §4 共 7 项
3. **DoD 验收动作**:`set-tier.sh` 置 plus 后实机展开真实档案推理链;录 10 月真实快照后看 2026 Q4 复盘(10 月首个快照落库后天然可做)
4. 以上完成 → prd-h 状态改「已完成」,票 05 resolved,期收口提交

## 9. 存量挂账核对

RISK-E-1(TLS)/ 金字塔(OUT-007,前置引擎扩展 ADR)/ gen-types.sh TS7(拍板项;本期 api-types 手工移植沿用,paths 段欠账未扩大)/ js-yaml audit fix / dev hydration 环境问题 / AC-10 caution 缺口 / P02 假设面板与情景推演(OUT-002,依赖模拟引擎)——全部延续,无新增除 §8 外。
