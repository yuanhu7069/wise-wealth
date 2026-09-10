# 架构实例：智策理财 · B 期（arch-v2.md）

> 关联规范：docs/base_line/基线-arch-Web域.md v0.2 / 基线-design-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260909-01 智策理财 B 期 Guide 首流程（prd-v1.md）
> 前序实例：docs/a/arch-a.md（A 期，v1；本文件为全局架构实例第 2 版，**增量式**：A 期已锁定内容不重述，仅记 B 期新增与变更）
> 数据库：外部 PostgreSQL dev 库（连接参数见 `.env`；无 TLS —— RISK-B-1，对外前必须解决）
> 上游总纲：docs/智策理财_PRD_V1.1.md；领域词汇：根 CONTEXT.md

---

## 0. 实例 ADR（B 期项目级决策）

> arch-a.md ADR-A-001~004 继续有效；ADR-A-001/002 本期**兑现**（见下）。基线 ADR-001~008 继续有效。

### ADR-B-001：模式库用 TOML 而非 YAML（产品 PRD §6.1 偏离）

【状态】已接受（2026-09-09，苑问经基线红线 1 审批流程批准）
【上下文】产品 PRD 写 YAML 时假设 Python 生态；Rust 侧 serde_yaml 2024 年被原作者归档停更，YAML 生态不健康。「加模式 = 加文件不改代码」的承诺与格式正交。
【决策】模式配置文件用 TOML（`toml` crate，Cargo 同源、活跃维护、支持注释），存于 `server/config/modes/`，**embed 进二进制**（include_str + 启动时解析缓存，模式库不依赖运行时文件系统）；chrono 同批批准（业务表 timestamptz 映射与 ISO 8601 序列化）。
【后果】正面：单二进制自包含；模式文件可注释可人工维护；解析错误编译期/启动期暴露。负面：产品 PRD「YAML」字眼需在 prd-v1 变更记录声明偏离（已记）。

### ADR-B-002：鉴权为单用户种子账号（兑现 ADR-A-001）

【状态】已接受（2026-09-09）
【上下文】ADR-A-001 承诺「B 期首个业务端点同批交付鉴权」。自用期多用户注册是过度设计。
【决策】单用户账号密码；凭证经 `SEED_USERNAME`/`SEED_PASSWORD` 环境变量由 `scripts/seed-user.sh`（调后端 CLI 子命令）写入/重置；argon2 哈希 + JWT（HS256，密钥 `JWT_SECRET` 环境变量，30 天期）HttpOnly Cookie；登录失败限流（内存滑动窗口，1 分钟 5 次）；**无注册/找回页**。
【后果】正面：每个业务端点服务端校验（基线 ADR-005 兑现），前端只是体验层。负面：遗忘密码 = 重跑种子脚本（可接受，RISK-B-2）；多用户升级时需加注册流与用户间数据隔离（届时另写 ADR）。

### ADR-B-003：方案版本化——重算生成新版本而非覆盖

【状态】已接受（2026-09-09）
【上下文】产品 PRD 追踪期要求「模式切换保留历史快照供对比」；覆盖式更新会让历史丢失，C 期补不回来。
【决策】`plans` 每次生成都插入新行（version 自增，事务内将旧版本 is_active=false、新版本 true，**单一 active 由部分唯一索引强制**——数据库层兜底，非仅应用层）。
【后果】正面：历史天然保留；C 期对比功能零迁移。负面：行数随重算增长（量级 §7.2 可忽略）。

### ADR-B-004：分账引擎为纯函数库（无 IO），金额全链路 i64 分

【状态】已接受（2026-09-09）
【上下文】引擎是产品核心资产（将来 Plus 推演复用同一内核）；CONTEXT.md 已定稿 inflow/余量桶/safety_first 语义；基线 ADR-004 红线（禁 float 金额）。
【决策】引擎落 `server/src/domain/`（纯结构体 + 纯函数，输入 = 档案 + 模式配置，输出 = 分账结果 + DecisionTrace[]；**零 IO、零 HTTP 感知**），`services/plan_service.rs` 只做编排（读档案 → 调引擎 → 持久化）。比例配置用**万分比整数**（3000 = 30%）；`÷24` 等除法统一 `(x + 12) / 24`（i64）+ 百元取整（RULE-012）后写单测。
【后果】正面：引擎可脱离数据库全量单测（金例 A/B/C 直接进 `cargo test`）；「一套内核两套视图」原则落地。负面：无（分层成本已由基线 ADR-007 承担）。

### ADR-B-005：埋点入库不上报（自用期链路验证）

【状态】已接受（2026-09-09）
【上下文】产品 PRD §12.2 要求 MVP 埋点，但自用期无第三方上报通道与漏斗看板。
【决策】`analytics_events` 表（PostgreSQL）；事件在后端/Server Action 触发（基线 §11.3）；payload 只存枚举与步号（金额永不入埋点，§11 日志脱敏）；写入失败 tracing::warn 后继续，不阻断主流程（RULE-019）。
【后果】正面：将来对外接上报通道时改写入端一点即可。负面：自用期无看板（OUT-008，可接受）。

---

## 1. 页面与路由映射（B 期增量）

| 页面ID | 页面名称 | Next 路由 | 组件类型 | 备注 |
|--------|---------|-----------|---------|------|
| P01 | 产品首页（改造） | `/`（`web/src/app/page.tsx`） | Server 取数（active 方案摘要）+ 摘要卡；页脚健康卡沿用 A 期 | 双形态；Token 样例区删除 |
| P02 | 登录页（新） | `/login` | Client（表单交互）+ Server Action 提交 | 已登录访问 → redirect P01 |
| P03 | 引导问卷（新） | `/questionnaire` | Server 壳（读草稿/档案）+ Client 向导 + Server Action 逐步保存 | 6 步状态机 |
| P04 | 方案页（新） | `/plan` | Server（读 active 方案；含 `loading.tsx` 骨架） | 五段静态渲染；`/plan/[version]` 查历史版本（P1，本期只做当前版） |

鉴权边界：`/`、`/questionnaire`、`/plan` 在 Next layout 层读取会话（Server Component 调后端 `/api/v1/auth/me` 或验 Cookie），未登录 → redirect `/login?from=`；`/login` 除外。**服务端强制在 Rust 侧**（§5），Next 层只是体验。

## 2. 核心实体与数据模型（首张业务表，兑现 ADR-A-002 备份前提）

金额列一律 `BIGINT` 分 + `_cents` 后缀；时间 `timestamptz`（UTC，chrono）；ID `UUIDv7`（uuid crate）；软删 `deleted_at`（本期无删除入口，列就位）。

| 表 | 关键字段 | 说明 |
|----|---------|------|
| `users` | id, username UNIQUE, password_hash, tier(DEFAULT 'free'), created_at | 单行（种子写入）；tier 为产品分层预留 |
| `profiles` | id, user_id FK, horizon, drawdown_response, income_stability, has_social_security, has_commercial_insurance, mortgage_balance_cents NULL, dependents, income_monthly_cents, expense_fixed_monthly_cents, savings_cents NULL, goal, questionnaire_completed BOOL, draft_step TINYINT, updated_at | 一行/用户，upsert；`draft_step` + 各答案列兼作草稿（0=未开始，6=完成）；RULE-004/005 的服务端校验即校此表入参 |
| `plans` | id, user_id FK, l1_mode, l2_mode, version INT, is_active BOOL, profile_snapshot_json JSONB, decision_traces_json JSONB, created_at | version 用户内自增；**部分唯一索引 `UNIQUE(user_id, version)` + `UNIQUE(user_id) WHERE is_active`**；快照冻结生成时的档案与推理链（Plus P1 数据源预埋） |
| `plan_buckets` | id, plan_id FK, bucket_id, name, amount_monthly_cents, target_amount_cents NULL, note TEXT | 每方案 3-4 行 |
| `analytics_events` | id, event_type, payload_json JSONB, occurred_at | §9.5 五事件；payload 仅枚举/步号 |

迁移：`0002_b_auth_and_users.up/down.sql`、`0003_b_profiles_plans_buckets.up/down.sql`、`0004_b_analytics_events.up/down.sql`——**结构迁移可回滚（DOWN 全写）**；`.sqlx/` 离线缓存照常提交。

## 3. 需求 → 能力映射（RULE → 代码落点）

| RULE | 后端实现位置 | 前端实现位置 |
|------|-------------|-------------|
| RULE-001（鉴权边界/from 白名单） | `api/middleware/auth.rs`（除 /health、/auth/login、/modes 外全校验）+ `dto` 层 from 校验（站内路径才透传） | Next layout + `lib/api.ts` 401 拦截 → `/login?from=` |
| RULE-002（登录限流） | `api/middleware/login_limiter.rs`（内存滑动窗口） | ERR-005 文案（errors.ts 映射 RATE_LIMITED） |
| RULE-003/004/005（问卷草稿/校验） | `services/profile_service.rs`（step upsert + DTO validator） | P03 向导组件（逐步 Server Action `saveQuestionnaireStep`） |
| RULE-006（主推） | `services/recommend_service.rs` | P03 步 6 双卡（`recommended` 标记来自后端） |
| RULE-007~014（分账引擎全集） | `domain/engine/`（纯函数）+ 单测金例 A/B/C | P04 只渲染引擎输出，**前端零计算** |
| RULE-015（L2 匹配） | `domain/engine/l2_match.rs` | P04 第二段展示匹配结果与原因标注 |
| RULE-016（版本化） | `services/plan_service.rs`（事务 + is_active 切换，数据库部分唯一索引兜底） | P01 摘要（active 版本）+ P04（当前版） |
| RULE-017（合规红线） | 引擎输出类型只有大类枚举（编译期保证无产品名） | 方案文案静态；grep 门禁（test-report §合规项） |
| RULE-018（金额精度） | i64 分 + 万分比（ADR-B-004） | `format-currency.ts`（千分位/2 位小数/tabular-nums 右对齐） |
| RULE-019（埋点不阻断） | `services/analytics_service.rs`（失败 warn 不抛） | - |

## 4. API 清单（B 期新增，均挂 `/api/v1`，统一信封）

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| POST | `/auth/login` | 登录签发 Cookie | 公开（限流） | `{username, password}` | `{user}` + Set-Cookie |
| POST | `/auth/logout` | 清 Cookie | 公开 | - | `{}` |
| GET | `/auth/me` | 会话探测 | 公开（无 Cookie → 401 信封） | - | `{user}` |
| GET | `/modes` | L1 模式列表（名称/理念/徽章/适用标签） | 公开（登录页前也可读？**否——需登录**，与 P03 步 6 一致） | - | `{items: Mode[]}` |
| GET | `/profiles/me` | 读档案（含草稿步号） | JWT | - | `Profile` |
| PUT | `/profiles/me/step` | 保存单步答案（upsert + draft_step 推进） | JWT | `{step, answers…}`（DTO 分步校验） | `{profile}` |
| POST | `/plans` | 生成方案（读完整档案 → 引擎 → 版本化落库） | JWT | `{l1_mode}` | `{plan, buckets, l2, traces}` |
| GET | `/plans/active` | 当前 active 方案 | JWT | - | 同上 |
| GET | `/health` | A 期不变 | 公开 | - | 不变 |

> codegen 链路不变：utoipa → openapi.json → api-types.ts；**每次接口变更后重跑 `scripts/gen-types.sh`**（web/scripts/）。

## 5. 权限校验点清单

| 校验点 | 位置 | 校验内容 | 越权/未认证时行为 |
|--------|------|---------|------------------|
| 中间件全路由校验 | `api/middleware/auth.rs` | JWT Cookie（除 login/logout/me/health 白名单） | 401 UNAUTHORIZED 信封 |
| login 限流 | `api/middleware/login_limiter.rs` | 1 分钟 5 次失败 | 429 RATE_LIMITED |
| from 参数白名单 | `dto/login_dto.rs` + Next 侧同校验 | 仅 `/` 开头且非 `//` | 丢弃 from，登录后落 `/` |
| profiles/plans 路径归属 | service 层（单用户下恒真，**结构就位**：查询一律带 user_id 条件） | - | 404 NOT_FOUND |

越权测试（基线 §14.1）：每个业务端点写一条「无 Cookie → 期望 401」用例；`from=//evil.com` 用例一条。

## 6. 外部依赖与集成（B 期增量）

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| toml crate（新批准） | 模式配置解析 | 解析失败 = 启动失败（模式库是编译期资产） | 无密钥 |
| chrono（新批准） | timestamptz / ISO 8601 | - | 无密钥 |
| jsonwebtoken + argon2 | 基线 §1.1 清单内 | - | `JWT_SECRET` 环境变量 |
| 外部 PG dev 库 | 业务数据 | /health db=error + 页面错误态（沿用 A 期） | `.env`（无 TLS，RISK-B-1） |

`.env.example` 新增键：`SEED_USERNAME` / `SEED_PASSWORD` / `JWT_SECRET`。种子脚本 `scripts/seed-user.sh` 调 `wise-wealth-server seed-user` 子命令（读环境变量，argon2 哈希后 upsert）。

## 7. 数据与迁移（B 期兑现 ADR-A-002）

- 迁移三份见 §2；**执行前先备份**（基线 §7.2 第 4 条）
- **备份策略（首个业务表落地，ADR-A-002 兑现）**：
  - `scripts/backup.sh`：`pg_dump --format=custom` → `backups/wise_wealth_$(date +%Y%m%d_%H%M).dump` + SHA256 校验文件；保留策略：日备 7 份 + 周备 4 份（脚本自动清理）
  - 备份位置：本地 `backups/`（.gitignore 排除）+ **异地一份**：苑问手动拷贝至云盘/另一机器（脚本完成后输出提醒）；恢复演练记录回填本节
  - **上线前演练**：AC-15 要求在副本库（`wise_wealth_backup_test`）恢复一次并 SQL 抽验 profiles/plans 行数与金额
- 红线自查：资金类数据（收入/存款）已入库 → **未完成 AC-15 演练前，苑问不录入不可重建的真实数据**（prd §11 红线 4）

## 8. 安全要点（B 期增量）

- 敏感字段：`password_hash`（永不返回 API）、`income_monthly_cents` / `savings_cents` 等财务列（仅 `/profiles/me`、`/plans*` 鉴权后返回）
- 日志脱敏：**金额、凭证、JWT、用户名不进日志**；埋点 payload 白名单（枚举/步号）；tracing span 只带 request_id + 路由名
- Cookie：HttpOnly + SameSite=Lax + `Secure` 生产启用（dev 允许 http，走 `APP_ENV` 分支）
- 合规：RULE-017 编译期保证（引擎输出为大类枚举，无字符串产品位）；文案 grep 门禁进 test-report
- 依赖审计：cargo audit + npm audit 复跑（test-report §安全审计）

## 9. 部署与启动（增量）

拓扑不变（A 期 §9）：WSL2 双服务 → 外部 PG。变更点：

- `scripts/start.sh` 增加步骤：迁移后自动执行**种子账号幂等写入**（已存在则跳过；`SEED_*` 缺失时打印提示但不阻塞服务启动——登录页会因无账号不可用，属 ERR-004 场景）
- `scripts/check.sh` 不变（新增用例自动纳入 cargo test）
- 环境分级不变：dev/prod 两库分离仍延后（RISK-B-1 同链）；**当前 dev 库即业务库**

## 10. 测试计划（基线 §14.3 + A 期 §10 模式）

**引擎单测（`cargo test`，金例锁定——ADR-B-004 的核心收益）**：

| 用例 | 断言 |
|------|------|
| 金例 A 四账户 | 四桶金额逐项 == §7.1 表；L2 == 60/40；traces 含 RULE-009/012 节点 |
| 金例 B 达标 | 备用 0 / 投资 3,900 / surplus 展示 52,800；无缺口节点 |
| 金例 C 不足 | 投资与备用均 0；inadequate_income 提示标志位 |
| 50/30/20（金例 A 档案） | 6,000/3,600/2,400；L3 文案触发条件正确 |
| 短久期（1 年内） | L2 现金 100% + 原因标注（RULE-015） |
| ÷24 舍入 | 缺口 73,200 → 3,100（百元取整 half-away-from-zero 边界用例） |
| 赡养上浮档 | 3→6、6→9、9→12、12 封顶 |
| 万分比 | 3000 == 30% 计算无浮点 |

**鉴权集成测试**：login 成功/凭证错/限流（第 6 次 429）；无 Cookie 访问 profiles/plans/modes → 401；from 白名单（`/plan` 通过、`//evil.com` 丢弃）。

**service 单测**：step upsert 幂等（重复提交同步不涨版本）；plan 版本化事务（生成两次 → version 1/2，active 唯一）；埋点失败不阻断（mock 注入错误）。

**前端手工清单**（对照 prd §8.3/8.4 与 design-v2）：AC-16 双端走查（375px + 桌面）、暗色三态切换、AC-6 断点恢复、Token 门禁、grep 产品名零命中（AC-13）。

**备份恢复**：AC-15 演练脚本化（backup → 副本库 restore → SQL 抽验），记录进 test-report-b.md。

**已知风险规避**：RISK-B-1（开发期接受，对外前冻结）；RISK-B-3（金例锁定 + 苑问真实数字校准）。

---

## 11. B 期交付物清单与验收顺序

1. `server/`：迁移 ×3 + 引擎（domain/engine + 金例单测）+ 鉴权中间件 + API ×9 + 种子子命令
2. `web/`：P02/P03/P04 新页 + P01 改造 + 暗色切换 + api-types 重生成
3. `scripts/`：seed-user.sh + backup.sh（含演练）
4. `docs/b/`：test-report-b.md
5. 验收顺序：引擎单测绿 → 鉴权链路（curl 无 Cookie 401）→ 双端页面走查 → 埋点 SQL 查验 → 备份演练 → 苑问真实数字端到端（DoD 验收动作）

**A 期遗留清理核对**：触控热区 44px（AC-16 一并验收）、Token 样例区移除（AC-16）、TS7 vendor 说明与 shadcn 手写组件维护说明 → 写入 README「已知工作态」节（本期一并补，test-report 核对）。
