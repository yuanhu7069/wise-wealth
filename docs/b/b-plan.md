# 执行计划：智策理财 · B 期（b-plan.md）

> 对应需求：REQ-20260909-01（prd-v1.md）／架构：arch-v2.md／设计：design-v2.md／原型：design-v1.html
> 生成日期：2026-09-09　状态：待执行（等苑问走查原型后启动）
> 执行原则：复刻 A 期模式——串行推进、每任务完成门不过不进下一任务、对照基线 §16/§13 红线自查
> 领域词汇：根 CONTEXT.md（引擎代码注释与测试名一律用词汇表术语）

---

## 0. 环境与前置事实（2026-09-09）

| 项 | 状态 |
|----|------|
| A 期骨架 | 已验收（信封/Token/BFF/codegen/健康端点/五态模式全量复用） |
| 外部 PG dev 库 | 可达、psql 认证通过；**无 TLS（RISK-B-1，开发期苑问已接受）** |
| 新依赖审批 | toml + chrono 经基线红线 1 流程已批（ADR-B-001）；jsonwebtoken/argon2 在基线清单内 |
| 种子凭证 | `.env` 待加 `SEED_USERNAME` / `SEED_PASSWORD` / `JWT_SECRET`（T2 前配置） |
| 遗留清理项（本期一并） | 触控热区 44px、Token 样例区移除、TS7 vendor 说明 + shadcn 手写组件维护说明进 README |

**启动门**：苑问走查 design-v1.html（四路径点通 + 状态循环演示）并确认 → prd-v1.md 状态改「已确认」→ 进入 T1。

## 1. 任务分解

### T1 · 数据层与迁移（arch-v2 §2）
1. `.env.example`/`.env` 增 `SEED_USERNAME`/`SEED_PASSWORD`/`JWT_SECRET`；`Cargo.toml` 增 toml/chrono/jsonwebtoken/argon2
2. 迁移 ×3（users / profiles+plans+plan_buckets / analytics_events），DOWN 全写；**执行前手动 pg_dump 一次**（基线 §7.2 第 4 条）
3. 部分唯一索引：`UNIQUE(user_id, version)`、`UNIQUE(user_id) WHERE is_active`
4. `cargo sqlx prepare` 更新 `.sqlx/` 并提交
- **完成门**：三迁移 up/revert/up 各跑一遍；clippy 零告警；`.sqlx/` 入库

### T2 · 鉴权链路（ADR-B-002，RULE-001/002）
1. `users` 读写 repo + `seed-user` CLI 子命令（读环境变量，argon2 哈希 upsert，幂等）
2. JWT 签发/校验（HS256，30 天）+ HttpOnly Cookie；`api/middleware/auth.rs` 白名单外全校验；登录限流（内存滑动窗口）
3. `POST /auth/login` `/auth/logout` `GET /auth/me`；from 白名单校验（DTO 层）
4. 越权测试：无 Cookie × 每业务端点 → 401；限流第 6 次 → 429；`from=//evil.com` 丢弃
5. `scripts/seed-user.sh`；`start.sh` 接入幂等种子步骤
- **完成门**：curl 全链路通（登录→Cookie→me→受限端点→logout）；限流用例绿；AC-1/2/4 后端侧通过

### T3 · 分账引擎（ADR-B-004，RULE-005~015，纯函数）
1. `server/config/modes/` 两份 TOML（four_accounts / 50_30_20，embed 进二进制）
2. `domain/engine/`：档案/模式结构体（金额 i64 分、比例万分比）→ 求解器（固定→pct→规则→余量 + safety_first）→ L2 匹配 → DecisionTrace[]
3. 单测全表：金例 A/B/C + 50/30/20 + 短久期 + ÷24 舍入边界 + 赡养上浮 + 万分比（arch-v2 §10 表逐条）
- **完成门**：**引擎单测先行全绿再写任何 HTTP 层**（TDD：金例即测试用例）；clippy 零告警；引擎模块零 IO（grep 无 sqlx/axum 引用）

### T4 · 业务 API 与服务层（RULE-003/006/016/019）
1. profiles：GET me / PUT step（分步 DTO 校验 + draft_step 推进 + upsert 幂等测试）
2. recommend service（RULE-006 主推规则）+ GET /modes（TOML 读取映射）
3. plans：POST（引擎编排 + 版本化事务 + 快照/trace 落库）、GET active
4. analytics service（5 事件，失败 warn 不抛）+ 各端点埋点调用点
5. OpenAPI 注解齐 → 重跑 gen-types → 前端 `tsc --noEmit` 暴露检查
- **完成门**：service 单测绿（步保存幂等/版本事务/埋点失败不阻断）；AC-5/8/12/14 的后端侧 curl 验证通过

### T5 · 前端四页 + 全局（AC-3/6/7/16/17）
1. P02 登录页（表单五态 + ERR-004/005/006 + from 回跳）；401 全局拦截（lib/api.ts）
2. P03 问卷向导（步进条/选项卡/数字输入组/推荐双卡；逐步 Server Action 保存；断点恢复；字段错误文案）
3. P04 方案页（五段 + 比例条 + format-currency + 骨架）
4. P01 改造（双形态摘要卡 + 空态 + **Token 样例区删除** + 页脚沿用）
5. 暗色手动切换（class 策略 + localStorage 偏好 + 跟随系统默认）；触控热区移动断点 44px；顶部栏
6. Token 门禁 + biome + tsc 全绿
- **完成门**：DevTools 375px 走通登录→问卷→方案；断点恢复实测（AC-6）；暗色三态切换无残留亮块；Token 门禁零命中

### T6 · 验收收尾（DoD 全项）
1. check.sh 全绿 + cargo audit / npm audit
2. **备份演练（AC-15，ADR-A-002 兑现）**：backup.sh → 副本库 restore → SQL 抽验 → 演练记录回填 arch-v2 §7 与 test-report-b.md
3. 苑问真实数字端到端走查（DoD 验收动作，含强制中断一次）+ AC-13 合规 grep（产品名零命中）
4. 埋点 SQL 查验（AC-14）；`test-report-b.md` 产出（AC 回链 + 基线 §16 清单 + A 期遗留清理核对）
5. prd-v1.md 状态改「已完成」；README 补 TS7/shadcn 工作态说明（arch-v2 §11）
- **完成门**：DoD 六项全过；苑问确认闭环

## 2. 执行顺序与依赖

```
T1 → T2 → T3 → T4 → T5 → T6
      └ T2/T3 可并行(鉴权与引擎互不依赖)——单执行者实际串行,T3 优先(TDD 金例先行)
```

关键顺序约束：**T3 引擎单测必须在 T4 API 之前全绿**（金例锁定是 RULE 精度的护栏）；T5 依赖 T4 的 api-types 重生成。

## 3. 交付物清单

- `server/`：迁移 ×3、`config/modes/` TOML ×2、`domain/engine/`、auth 中间件与 API、seed 子命令、`.sqlx/` 更新
- `web/`：P01-P04、暗色切换、401 拦截、api-types.ts 重生成
- `scripts/`：seed-user.sh、backup.sh（含保留策略与 SHA256 校验）
- `docs/b/`：test-report-b.md；prd-v1.md 状态流转
- 根：README 工作态说明更新

## 4. 验收标准回链（17 AC → 任务）

| AC | 覆盖任务 | 验证方式 |
|----|---------|---------|
| AC-1/2/4 登录与限流 | T2 | curl 集成测试 |
| AC-3 会话过期回跳 | T5 | 浏览器实测（改 Cookie 过期） |
| AC-5 金例 | T3+T5 | 单测 + 页面金额逐项比对 |
| AC-6 断点恢复 | T5 | 实测中断重进 |
| AC-7 字段校验 | T4/T5 | 单测 + 页面文案 |
| AC-8 推荐双卡 | T4/T5 | 金例 A 主推四账户 |
| AC-9 短久期 | T3 | 单测（现金 100% + 原因标注） |
| AC-10 达标 | T3 | 金例 B 单测 |
| AC-11 收入不足 | T3 | 金例 C 单测（无负数断言） |
| AC-12 版本化 | T4 | 生成两次事务断言 |
| AC-13 合规 grep | T6 | 文案/配置全量 grep |
| AC-14 埋点 | T4/T6 | SQL 查验 |
| AC-15 备份演练 | T6 | 演练记录 |
| AC-16 双端/暗色/样例区移除 | T5/T6 | 375px + 桌面走查 |
| AC-17 金额格式 | T5 | 全站抽查 |

## 5. 风险与执行注意

- RISK-B-1（无 TLS）：开发期已接受；**对外前冻结线不变**
- RISK-B-3（规则表未经校准）：T3 金例锁定 + T6 苑问真实数字校准；发现不合理调表不改引擎结构
- T3 ÷24 舍入实现注意：(缺口分 + 1200) / 2400 的整数除法 + 百元取整方向测试（half away from zero）
- T5 移动端主操作固定底部安全区（design-v2 §1.4）；P03 步切换为客户端状态零网络往返（prd §9.1）
- 全程金额红线（基线 ADR-004）：T3/T4 评审时 grep `f64` 参与金额即打回
