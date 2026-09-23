# 04 · P05 落地：`/tracking` 页实现 + T2 验收

Status: resolved
Labels: ready-for-agent

> e-plan T4。前置：03 固化完成。规格：arch-e.md §1/§3；prd-e §8（五态矩阵/错误清单）。

## 步骤

1. `web/src/app/tracking/`：Server 壳（`GET /snapshots` 取 items+summary）+ Client 录入卡（Server Action 提交 PUT，覆盖走确认条 ERR-E-03）+ 偏离提示条（涨/跌变体）+ 距离感进度（文本承载，达标态绿变体）+ 历史列表 + 导出区（`<a>` 走 lib/api 代理）
2. 五段结构与视觉对齐固化产物；token 只引契约值（红线 13）；中文字重/行高/字距等沿 D 期既定偏离直接套用（spec §4）
3. 动态接线（spec §4 静态值接动态，非偏离）：桶名/桶数来自 `/plans/active`；数字来自 summary；导出禁用态来自快照计数
4. 空态/错误态接线：无方案 → EMPTY-E-01 引导；无快照 → EMPTY-E-02；提交失败保留输入（ERR-E-01）；401 回跳 `/login?from=/tracking`；库不可达错误态（AC-16）
5. 「本月特殊」勾选与「跳过本月」按钮（触发 `snapshot_skip` 客户端埋点）；删除入口（仅最新月，二次确认 ERR-E-04）
6. **T2**：实现 vs 固化产物逐项核对，偏离记 `.scratch/e-tracking/spec.md` §4 清单；清单外不得自行发挥
7. 站点页脚沿 RULE-020/031（一行简述 + 展开）

## DoD

- `bash scripts/check.sh` 全绿（clippy/cargo test/biome/tsc/Token 门禁）
- AC-1/2/3/5/6/12/13/14/15 对应走查过（桌面 + 375px + 亮/暗）
- 调整清单草案落 spec §4 并交苑问拍板

## Comments

- 2026-09-23 · AI · 完成。server 壳三态(ready/no-plan/error,无方案不重定向、自承载整页空态)+ 五段视图(server)+ 录入卡客户端岛(实时千分位预览纯字符串运算、覆盖确认、跳过月)+ 删除按钮(仅首行)+ CSV 下载代理 `/api/export/[kind]`(Route Handler,红线 16)。T2 核对:清单外零偏离,6 项偏离/扩展全录 spec §4(删除/跳过为产品要求的功能扩展;`--color-success-subtle` 系产物合同既有值补注册)。
- 2026-09-23 · AI · **实机冒烟(只读+可回滚)**:登录 → P05 渲染五段(真实方案桶名,首月形态,导出置灰)→ PUT 2026-09(元转分正确,plan_version 冻结,首条 deviations=null,缺口 5.6 个月自洽)→ DELETE 回空态 → 历史月删除 422「只能删除最新一个月的快照」→ 残留 0。CSV 代理:attachment 文件名 + text/csv + BOM 首三字节 ✓。未登录 200 软重定向壳零内容泄漏(Next 16 机制,与既有四页一致)。
- 2026-09-23 · AI · 过程修两处:① page.tsx 三态判定被「默认 error 初始化」吃掉 ready 分支(有方案用户全落错误页)——重构为 flags 判定;② 手工 psql 建表未登记 `_sqlx_migrations` 导致启动迁移冲突——DROP 后 `sqlx migrate run` 正规应用(版本 7 已登记)。教训入账:迁移一律走 `sqlx migrate run`,不手工 psql。失败过的 `cargo sqlx prepare` 会清缓存,check.sh 离线门禁依赖它,已在建表后重新生成并补提交(`f5c0743`)。
