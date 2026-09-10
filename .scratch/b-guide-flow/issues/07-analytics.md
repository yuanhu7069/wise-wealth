# 07: 埋点链路（5 事件）

**What to build:** 走完一次完整流程后，可以在数据库里查到 5 类事件：页面访问、问卷开始、每步完成（带步号）、问卷完成、方案生成（带模式与版本号）。埋点只入库，不接第三方；它坏了不能影响主流程。

**Blocked by:** 05（问卷与方案流程都要存在，事件才有地方可埋）

**Status:** done（2026-09-11）

**完成记录**：新增 `analytics_events` 表（迁移 `0006`，列照 arch-v2 §2）与 `services/analytics_service.rs`（RULE-019 的落地文件）。三件事值得单独记：

1. **金额永不入埋点靠类型，不靠人记得**：事件只能是 `Event` 枚举的变体，每个变体自带的字段只有页面 id / 步号 / 模式 id / 版本号 —— 调用点没有可以塞金额的位置。载荷键白名单由单测逐条锁定（新增变体必须先回答「这是不是敏感数值」）。
2. **`record()` 返回 `()`**：调用方拿不到错误，也就不可能把它变成主流程的失败；写失败只 `tracing::warn`。失败路径由单测（指向不可达地址的池）断言「正常返回」，不靠 review 目测。
3. **`/analytics/events` 是本 ticket 对规格 API 表的补全**（spec.md 与 arch-v2 §4 已同步标注）：页面触达与问卷开始的触发点是 Next 侧的**服务端渲染**，渲染够不到数据库，必须有一个上报入口。入口白名单只放行前端才知道的 `page_view` / `questionnaire_start`；后端自己知道的三类（每步保存 / 问卷答全 / 方案生成）直接在 handler 入库 —— 同一事实开两个来源，迟早重复计数。前端上报走 `after()`（响应发出后才发请求），把「埋点可能慢 5 秒」与「页面什么时候出来」彻底解耦。

**两处对 prd-v1 §9.5 字面的明确偏离（实现期判定，记此备查）**：①`questionnaire_step_completed` 的步号**只可能到 5**——步 6 是推荐位、不落库，故「到达步 6」由 `questionnaire_completed` 表达（ticket 原文写 1-6，属对问卷步号的描述，非可实现范围）；②`questionnaire_completed` 的触发收窄为「**未答全 → 答全**」的跃迁，而非字面的「步 5 校验全部通过」——跳步只答步 5 也能过自己那步的校验，但那不叫问卷完成（与 `is_complete()` 同一判据，即「跳过步骤不该标记完成」，ticket 03 已锁）。两条都由单测锁住。

**端到端实测（`.scratch/b-guide-flow/tmp-verify-07.sh`，curl + psql 断言，15 节全通过）**：重置到「还没答问卷、没有方案」→ 请求前端 P01/P03/P04（带会话 Cookie，走真实服务端渲染）→ 埋点表里出现 `page_view{p01,p03,p04}` 与 `questionnaire_start`；步 1 保存 → `step_completed{1}`，答完步 2-5 → 步号 1-5 各一条 + `questionnaire_completed` **恰一条**；重复保存步 5 不再产生第二条完成事件；生成方案 → `plan_generated{l1_mode:four_accounts, plan_version:1}`，重新生成 → `plan_version:2`；**无方案时访问 P04 会跳回首页，不留 p04 触达**；`GROUP BY event_type` 五类齐全（验收动作）；载荷键白名单外 0 条、无 ≥100 的数值，而同库 `plan_buckets` 有 4 条金额（反向证据：不是碰巧没数字）；清空档案后再造一次「无草稿进入」→ `questionnaire_start` 仍为 1（一次性事件去重）。拒收侧：客户端上报 `plan_generated` / 缺 `page_id` / 未知 `page_id` / `questionnaire_start` 带 `page_id` 均 422 信封，无 Cookie 401。

**质量门**：后端 94 项单测全绿（新增 7 项：载荷逐条锁定、载荷键白名单、客户端白名单、四种拒收、完成跃迁、跳步不跃迁、失败不阻断），`scripts/check.sh` 五道门全绿；`web/src/lib/api-types.ts` 已按新端点重生成（arch-v2 §4「每次接口变更后重跑」）。

**遗留**：①`openapi.rs` 的注册面只补了本 ticket 的端点，`/profiles/me`、`/profiles/me/step`、`/modes`、`/plans`、`/plans/active`、`/auth/*` 六个端点自 02-06 期起只有 `#[utoipa::path]` 注解、未挂进 `ApiDoc`（`api-types.ts` 因此只含 health 与新端点）——**存量缺口，建议单列一张 ticket 补齐**，本 ticket 不顺带做；②「问卷开始」为一次性事件的判据是「表里还没有第一条」，自用期（单用户、档案无删除入口）与「首次」等价；多用户时该判据要加 user_id 维度；③前端人工走查（浏览器里的真实点击链路）归 ticket 09。

**切分理由：** 单列而非散在各 ticket 里埋的理由：埋点是**一层横切能力**（表 + 服务 + 调用点），集中做完可用一条 SQL 验证；若分散，每张 ticket 的验收都要重复验证同一张表。阻塞于 05 是因为 5 个事件里有 2 个只有方案流程跑通后才存在。

- [x] 5 类事件均可入库：页面访问 / 问卷开始 / 每步完成（带步号 1-5，见完成记录偏离①） / 问卷完成 / 方案生成
- [x] 方案生成事件载荷含模式 id 与版本号（`{"l1_mode": "...", "plan_version": N}`）
- [x] 写入失败不阻断主流程，仅记录告警日志（后端单测断言 + 前端 `after()` 上报只记 console 警告）
- [x] 事件载荷不含任何金额数值（类型层禁止 + 键白名单单测 + 全表 SQL 抽查）
- [x] 可用 SQL 直接查验事件类型分布与各步到达情况（`GROUP BY event_type`；步号分布见实测脚本第 8 节）
