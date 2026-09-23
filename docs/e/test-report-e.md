# E 期测试报告(test-report-e)

> 需求:REQ-20260922-01(prd-e.md)· 架构:arch-e.md · 设计:design-e.md
> 状态:**终稿**——苑问人工走查通过(2026-09-23),期收口
> 自动门禁最终状态:2026-09-23 `scripts/check.sh` 五步全绿(clippy -D warnings / cargo test 134 / biome / tsc / Token 门禁;评审修复提交 `a7df41c` 后复跑)

---

## 1. 自动门禁与红线

- `scripts/check.sh` 全绿(最终确认于票 05 提交 `7b5c717` 后复跑)
- arch 基线红线 19 条逐项自查:✅(金额 i64 分零 float / 无新增依赖超出批准面[chrono 承 ADR-B-001] / 凭证零硬编码 / 服务端权限校验[新端点全落 JWT 中间件] / 错误不吞 / 技术细节不回前端 / 日志无金额与凭证 / 迁移 0007 配套 / 列表分页[limit 默认 24,上限 120] / 无 println 残留 / Token 门禁通过 / .env 未入库 / 备份策略在并已演练 / 前端经 lib/api 代理 / codegen 产物未手改 / 无本机中间件探测 / 核心页过自动扫描[dev 走查 + biome a11y 规则])
- 合规:引擎与导出内容仍只到大类;P05 页脚一行简述 + 完整声明(RULE-020/031)

## 2. 测试构成与结果

| 层 | 数量 | 说明 |
|---|---|---|
| 域纯函数金例(tracking + csv) | 23 | deviations(超阈/达阈边界/基准回溯/前值≤0/透支/同名桶/自定义阈)· emergency_gap(3.2 金例/半量进位/达标/坏输入)· persisted_months(断月不清零)· csv(前缀拦截/引号倍增/BOM+CRLF/纯数字旁路) |
| dto 解析与校验 | 5 | 元转分金例(0.29 防 float)/非法金额逐类/月份日历 |
| service 纯函数 | 5 | 桶集合校验(缺桶/多桶分别指出)/应急桶选择(规则桶→投资桶回落) |
| config | 3 | 偏离阈值缺省/自定义/越界拒绝 |
| analytics 扩展 | — | 事件名与载荷逐条锁定;客户端白名单(snapshot_skip + p05);金额键位钉死 |
| 路由集成 | 3 | 新端点无 Cookie 401 ×5;未来月 422(不触库);非法月份 422 |
| **合计** | **134** | `cargo test` 全绿 |

## 3. 设计闸门(基线 T0/T1/T2)

- **T0**:四件套齐备并入库(github / redesign-existing-projects / high-fidelity / `.scratch/e-tracking/custom-instructions.md` @ `4f480bc`);苑问授权后生成
- **T1**(run `0c16a225`,7.7 min,一次过闸):A 绑定契约 56 项逐值全中、全文档 hex 超出冻结契约 0 个;B 自动项全过(自包含/tabular×5/焦点环/44px×10/role=status·alert×5/reduced-motion/aria-busy)。人眼项(亮/暗 × 375px)按 D 期批量节奏并入期末统一复核,苑问 2026-09-23 按「现方案通过」确认
- **T2**(票 04):清单外零偏离;偏离/扩展 6 项录 `.scratch/e-tracking/spec.md` §4(删除与跳过为产品要求的功能扩展;`--color-success-subtle` 系产物合同既有值补注册)
- 生成记录:`docs/design/生成记录.md` P05 行;快照 `docs/design/e-tracking/p05-tracking-github.html`(sha256 前 16 位 `a8b1142185d252f5`)

## 4. 实机走查(票 06 · 2026-09-23 苑问,**通过**)

- [x] 登录 → P01 追踪卡两态(未打卡提示/已打卡摘要)→ 去打卡 → P05
- [x] P05 五段走查(录入/进度/偏离/历史/导出)× 亮/暗 × 375px
- [x] 录入:预览千分位 / 覆盖确认 / 本月特殊 / 提交失败输入保留
- [x] 导出:双 CSV 下载(文件名/BOM/Excel 中文)
- [x] 删除最新月(二次确认)与历史月不可删
- [x] **DoD 验收动作**:本月真实数字打卡 1 条——**已达成**(snapshots 表 2026-09 一条,2026-09-23 04:11 录入;次月复访完成第 2 条后闭环全满,属跨月自然验证,不阻塞期收口)

## 5. 埋点 SQL 查验(2026-09-23,已过)

8 类事件齐备;冒烟产生的 `snapshot_submit`(载荷仅 is_overwrite/is_special)/`snapshot_delete`/`export_csv`/`page_view p05` 均入库,**金额零入载荷**。

## 6. 备份恢复(2026-09-23,已过)

`restore-drill.sh` 增项复跑:备份 → 副本库恢复 → 12 项抽验(含新增 snapshots 行数/余额合计)全部一致 → 副本库起服务 db=ok + 种子账号可登录 + 可读方案(第 8 版)。prod 分支演练仍挂账(B 期 I-9,不阻塞 dev 验证)。

## 7. 安全审计(2026-09-23)

- cargo audit(离线库):仅命中已知 **rsa Marvin Attack medium**(RUSTSEC-2023-0071,B 期 I-7,上游无修复,继续挂账)——**无新增漏洞**
- npm audit:本机网络无法访问审计端点(项目已知网络受限,同 shadcn CLI 先例),未能执行;本期前端**零新增依赖**,暴露面不变
- TLS RISK-E-1:维持 2026-09-22 拍板(自用期接受,对外前必须解决)

## 8. 已知偏离与遗留

- 工具链:`gen-types.sh` 在 TS7 环境崩溃(TS7 无 ts.factory),本次以 vendor ts5.9.3 隔离环境生成;**长期修法待苑问拍板**(工单 02 Comments)
- 历史快照分页 UI:首屏近 24 个月 + 如实计数说明 + CSV 兜底;完整分页 UI 待量级需要(12 条/年)
- 次月复访:DoD 第二条快照由苑问 2026-10 月底打卡自然完成
- I-8 收入不足文案歧义、I-9 prod 恢复演练、I-12 非 Chromium 走查:继续挂账(非本期域)

## 9. 评审与修复记录

代码评审(review since `4f480bc`)10 项发现,**全部处置**,2026-09-23 修复提交并实机复验:

| # | 级别 | 发现 | 处置 |
|---|------|------|------|
| 1 | **规范违例** | PUT 未限制当月:历史快照可被改写并重挂 plan_id(RULE-028 要求的 service 校验缺失) | service 加当月门槛(`MonthNotAllowed`),历史月/未来月一律 422;实机复验「PUT 2026-08 → 422」 |
| 2 | UX bug | 覆盖预填漏还原「本月特殊」标记 | EntryCard 增 `initialSpecial`,refresh 后跟随 |
| 3 | UX bug | 删除后表单残留旧值,一键原样写回(击穿 RULE-029 恢复口) | 渲染期重置模式(initial 序列化基准变化即重置) |
| 4 | 一致性 | 「当前月」三处各自计算,跨机时区在月界会互相矛盾 | API 响应下发后端权威 `current_month`;P05/P01 均不再自算 |
| 5 | 边界 | 并发同月首插撞 UNIQUE → 500 | INSERT 带 ON CONFLICT DO UPDATE;xmax 如实区分插入/覆盖(埋点不谎报) |
| 6 | 数据损坏 | 导出双重转义(桶名含逗号/前缀符时内容被翻倍引号) | service 交原始字段,转义统一在 render_csv |
| 7 | 校验不一致 | 前端预览放行「3200.」而后端拒绝,按钮状态与提交结果相左 | 前端正则与 `parse_yuan_to_cents` 同一严格度 |
| 8 | 语义 | 最新快照非当月时,偏离条仍说「本月」 | 按快照所属月标注 |
| 9 | 性能 | 列表 N+1(每行 by_id 取版本)+ summary 重复查询 | list JOIN 版本号随行;to_view 纯化 |
| 10 | 产品 | 首屏仅近 24 个月且无分页,计数与可见行数矛盾 | 增如实说明(共 N 个月/显示 M 个月/更早走导出);完整分页 UI 量级不到(12 条/年)记遗留 |

修复后:`check.sh` 全绿;实机复验(历史月 422 / 当月 200 / current_month 下发 / 清理零残留)。
