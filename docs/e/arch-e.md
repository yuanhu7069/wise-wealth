# 架构实例：智策理财 E 期 · 追踪模块（arch-e.md）

> 关联规范：docs/base_line/基线-arch-Web域.md v0.2 / 基线-design-OpenDesign-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260922-01 智策理财 E 期追踪模块（prd-e.md）
> 前序实例：docs/b/arch-v2.md（全局架构实例第 3 版，**增量式**：A/B 期已锁定内容不重述，仅记 E 期新增与变更）
> 数据库：外部 PostgreSQL dev 库（连接参数见 `.env`；无 TLS —— RISK-E-1，2026-09-22 苑问拍板自用期接受，对外前必须解决）
> 上游总纲：docs/智策理财_PRD_V1.1.md（§4.4 追踪）；领域词汇：根 CONTEXT.md「追踪」节（本期新增六术语）

---

## 0. 实例 ADR（E 期项目级决策）

> ADR-A-001～004、ADR-B-001～005 继续有效；本期无推翻。

### ADR-E-001：快照按「用户 × 自然月」唯一，余额以 JSONB 冻结桶集

【状态】已接受（2026-09-22，拷问 Q7/Q11 定稿）
【上下文】产品 PRD §4.4 口径是「每月一次快照」；桶集合随方案版本变化（3 桶或 4 桶，切换 L1 模式后完全不同），关系列无法表达动态桶集；同月多条快照无业务语义（RULE-021 覆盖式）。
【决策】`snapshots` 表以 `UNIQUE(user_id, month)` 强制每用户每月一条（数据库层兜底，同 ADR-B-003 立场）；`month` 存当月 1 日（DATE）；`balances JSONB`（键 = bucket_id，值 = 分）+ `plan_id FK` 冻结提交时的方案版本。覆盖 = UPDATE（`updated_at` 审计），删除 = 当月硬删。
【后果】正面：按月查询/列表天然有序；历史快照不可变（RULE-028）由「只允许改当月」的 service 校验承载。负面：JSONB 无关系级约束——桶集合合法性由服务端校验兜底（对照提交时 active 方案的桶集，多桶/少桶/未知桶一律 422，见 §5）。

### ADR-E-002：偏离与进度是域层纯函数（`domain/tracking.rs`），前端零计算

【状态】已接受（2026-09-22，承 ADR-B-004 内核立场）
【上下文】偏离基准回溯（跳过特殊快照）、同名桶比对、前值 ≤ 0 排除（RULE-024/025）、缺口月数（RULE-026）都是可判定的业务口径，将来 Plus「真实 vs 计划对比」复用同一内核；B 期已确立「引擎纯函数 + service 编排 + 前端只渲染」。
【决策】新建 `server/src/domain/tracking.rs`：输入 = 快照序列 + active 方案（桶集、应急目标、必要月支出），输出 = `TrackingSummary { persisted_months, emergency{...}, latest_deviations }`；零 IO、零 HTTP 感知，金例单测锁定（§10）。
【后果】正面：口径唯一实现，P01 追踪卡与 P05 同源；「一套内核两套视图」延续。负面：无（分层成本已由基线 ADR-007 承担）。

### ADR-E-003：偏离阈值走环境变量，不进 UI 不进库

【状态】已接受（2026-09-22，拷问 Q9 定稿）
【上下文】RISK-B-3 立场：规则参数要可调且不改引擎结构；OUT-006 明确本期不做 UI 调整入口。
【决策】`SNAPSHOT_DEVIATION_THRESHOLD_BP`（万分比，默认 2000 = 20%）由 `config.rs` 读取，进 `AppConfig` 传参给 tracking 纯函数；键名补入 `.env.example`（含注释）；缺省即默认值，缺键不报错。
【后果】正面：改阈值 = 改 `.env` 重启，零迁移零 UI。负面：多用户时需改为每用户配置（届时另写 ADR，本期单用户不预埋）。

### ADR-E-004：CSV 导出走专用端点直出 `text/csv`，手拼转义、不引新依赖

【状态】已接受（2026-09-22，承基线红线 1 依赖审批）
【上下文】统一信封（基线 §6.1）服务于前端 JSON 消费；文件下载是浏览器语义。导出列全部受控（数字 / 枚举 / 模式侧受控名称），CSV 结构简单。
【决策】`GET /snapshots/export` 与 `GET /plans/active/export` 响应 `Content-Type: text/csv; charset=utf-8` + `Content-Disposition: attachment`，正文 UTF-8 带 BOM（Excel 中文兼容）；字段转义手写函数（包裹引号 + 倍增引号 + 首字符为 `=`/`+`/`-`/`@` 时前缀 `'` 防公式注入）并配单测；**不引入 csv crate**（红线 1：免审批成本，转义逻辑单测覆盖）。
【后果】正面：无新依赖；导出语义不走信封是记录在案的**显式例外**（前端不以 JSON 解析这两个端点）。负面：若将来导出列扩展为用户自由文本，转义函数必须先补 fuzz 用例（届时评估 csv crate）。

---

## 1. 页面与路由映射（E 期增量）

| 页面ID | 页面名称 | Next 路由 | 组件类型 | 备注 |
|--------|---------|-----------|---------|------|
| P05 | 追踪页（新） | `/tracking` | Server 壳（读快照/方案 + summary）+ Client 录入卡（表单交互，Server Action 提交） | 生成走 design-e 三道闸门；五段纵排：录入卡 / 距离感进度 / 偏离提示条 / 历史列表 / 导出区 |
| P01 | 产品首页（改造） | `/` | Server 取数追加 snapshot summary | 摘要形态追加追踪卡；**代码级改动，不回炉生成**（基线：落地后真源 = 代码库） |
| P02/P03/P04 | - | - | - | 零改动 |

鉴权边界：`/tracking` 进 B 期 layout 层未登录重定向链（`/login?from=/tracking`）；服务端强制仍在 Rust 中间件（新端点全部落 JWT 校验，白名单**零变更**）。

## 2. 核心实体与数据模型（E 期增量）

延续 B 期约定：金额 `BIGINT` 分 + `_cents` 后缀；时间 `timestamptz`；ID `UUIDv7`。

| 表 | 关键字段 | 说明 |
|----|---------|------|
| `snapshots`（新） | id, user_id FK, plan_id FK（提交时方案版本）, month DATE（当月 1 日）, balances JSONB（bucket_id → 分）, special_month BOOL NOT NULL DEFAULT false, created_at, updated_at | **UNIQUE(user_id, month)**（部分/普通唯一均等价于此处语义，取普通唯一）；索引 `(user_id, month DESC)`；`plan_id` 不设外键级联删除（方案版本永存，ADR-B-003） |

校验落点：balances 的键集合必须 == 提交时 active 方案的桶集（service 层校验，422）；month 不可为未来月、格式 YYYY-MM。

迁移：`0007_snapshots`（UP/DOWN 全写，结构可回滚）；`.sqlx/` 离线缓存照常提交。

## 3. 需求 → 能力映射（RULE-021 起 → 代码落点）

| RULE | 后端实现位置 | 前端实现位置 |
|------|-------------|-------------|
| RULE-021（全桶必填 / 同月覆盖） | `services/snapshot_service.rs`（upsert：桶集校验 + UNIQUE 冲突转 UPDATE） | P05 录入卡 + 覆盖确认条（ERR-E-03） |
| RULE-022（精度/负数/上限） | `dto/snapshot_dto.rs`（validator） | 输入校验 + 等宽千分位预览（`format-currency.ts` 沿用） |
| RULE-023/024（跳过月 / 本月特殊） | `special_month` 列；跳过月 = 无行（无需代码） | 「本月特殊」勾选；「跳过本月」按钮（客户端埋点 `snapshot_skip`） |
| RULE-025（偏离口径） | `domain/tracking.rs::deviations()`（基准回溯 + 同名桶 + 前值 > 0） | P05 偏离提示条（涨/跌文案变体），**前端零计算** |
| RULE-026（缺口月数） | `domain/tracking.rs::emergency_gap()`（输入取 `plan_buckets.target_amount_cents` 与模式必要桶月转入合计，零新口径） | 距离感进度条（文本「还差 X 个月」承载，不只靠图形） |
| RULE-027（已坚持月数） | `domain/tracking.rs`（有快照月累计，跨断月） | 同上 |
| RULE-028（版本冻结） | 快照写入时冻结 `plan_id`；summary 计算取最新快照 × active 方案 | 历史列表展示各快照所属版本 |
| RULE-029（仅最新月可删） | `services/snapshot_service.rs::delete_latest`（month == 最新月才执行） | 删除入口 + 二次确认（ERR-E-04） |
| RULE-030（导出） | `api/v1/snapshots.rs` 导出端点 + `csv_escape()`（ADR-E-004） | 导出按钮组（`<a>` 走 lib/api.ts 代理） |
| RULE-031（免责声明） | - | P05 站点页脚（RULE-020 组件复用） |
| 埋点（§9.5） | `services/analytics_service.rs`：服务端事件 `snapshot_submit`/`snapshot_delete`/`export_csv`；客户端白名单扩 `snapshot_skip` + `PageId::P05` | page_view p05（既有上报组件） |

顺手修复（B 期存量缺口）：将 profiles / plans / modes 六个已注解未注册的端点补挂进 `openapi.rs` 的 `ApiDoc`，恢复 codegen 链路完整性；本期待遇与新增端点同批验证（`gen-types.sh` 产物含六端点类型）。

## 4. API 清单（E 期新增，均挂 `/api/v1`）

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| GET | `/snapshots` | 快照列表 + 追踪摘要 | JWT | `?limit=24&offset=0`（默认近 24 月，红线 11） | `{items[], summary{persisted_months, emergency{target_cents, balance_cents, gap_months, met}, latest{month, deviations[]\|null}}}` |
| PUT | `/snapshots/:month` | 录入/覆盖（upsert） | JWT | `{balances{bucket_id: cents}, special_month}` | `{snapshot, deviations\|null}` |
| DELETE | `/snapshots/:month` | 删除（仅最新月） | JWT | - | `{}`（非最新月 → 422） |
| GET | `/snapshots/export` | 快照 CSV 长表 | JWT | - | `text/csv`（ADR-E-004，信封例外） |
| GET | `/plans/active/export` | active 方案 CSV | JWT | - | 同上 |
| POST | `/analytics/events` | 既有端点，白名单扩 | JWT | `snapshot_skip` / `page_view p05` | 不变 |

参数校验：`month` 格式 `YYYY-MM` 且 ≤ 当前月；PUT 幂等（同月重复 PUT = 覆盖，语义同 B 期 step upsert）。codegen 链路不变：新增/变更后重跑 `scripts/gen-types.sh`。

## 5. 权限校验点清单（E 期增量）

| 校验点 | 位置 | 校验内容 | 越权/未认证时行为 |
|--------|------|---------|------------------|
| 中间件全路由校验 | `api/middleware/auth.rs` | 新端点 5 个全部落入 JWT 校验（白名单零变更） | 401 UNAUTHORIZED 信封 |
| 桶集合一致性 | `snapshot_service.rs` | balances 键集 == active 方案桶集 | 422 VALIDATION_ERROR |
| month 合法性 | `dto/snapshot_dto.rs` | YYYY-MM、≤ 当前月 | 422 |
| 删除边界 | `snapshot_service.rs` | 目标月 == 该用户最新快照月 | 422（历史月不可删） |
| 路径归属 | service 层 | 查询恒带 user_id（B 期模式，结构就位） | 404 |
| 埋点白名单 | `analytics_service.rs` | `snapshot_skip` 入客户端白名单；page_id 枚举 +P05 | 422 |

越权/降级测试：新端点各一条「无 Cookie → 401」；`month=2099-01` → 422；PG 停机 → P05 错误态（AC-16）。

## 6. 外部依赖与集成（E 期增量）

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| Open Design daemon 0.22.1（127.0.0.1:7456） | P05 界面生成（**生成期一次性**，运行时零依赖） | 连不上 → 停下报告苑问，不手搓、不换来源（基线 §2.2） | BYOK（用户侧） |
| 其余 | 无新增 crate / 中间件 | - | `SNAPSHOT_DEVIATION_THRESHOLD_BP` 进 `.env.example`（可选键，缺省 2000） |

`.env.example` 本期新增键仅上述 1 个；无新脚本。

## 7. 数据与迁移（E 期增量）

- `0007_snapshots` 单迁移；**执行前先备份**（基线 §7.2，`scripts/backup.sh` 已就位）
- 备份与恢复演练增量：`restore-drill.sh` 的抽验清单追加「snapshots 行数 + 金额合计」一项；演练复跑一次并回填 test-report-e（B 期 I-9 的 prod 分支演练仍挂账，不阻塞本期 dev 验证）
- 红线 15 自查：快照属资金类数据，备份策略已存在 ✅；dev/prod 库分离继续延后（与 RISK-E-1 同链，对外前一并处理）

## 8. 安全要点（E 期增量）

- 敏感字段：`balances`（余额）仅鉴权端点返回；**金额、余额不进日志与埋点**（tracing span 只带 request_id + 路由名，B 期立场）
- CSV 公式注入：`csv_escape()` 转义 + 单测（ADR-E-004）
- 输入面：month / balances 全量 DTO 校验；JSONB 无库级约束 → 服务端桶集校验兜底（ADR-E-001）
- 导出文件名：`snapshot_export_YYYYMMDD.csv` / `plan_export_YYYYMMDD.csv`，不含用户名等敏感词

## 9. 部署与启动（增量）

拓扑不变（WSL2 双服务 → 外部 PG）。`start.sh` 不改（迁移自动执行 0007）；`check.sh` 不改（新增用例自动纳入 cargo test）。环境分级不变：dev 库即业务库，分离继续延后。

## 10. 测试计划（基线 §14.3 模式，承接 B 期）

**域纯函数单测（`cargo test`，金例锁定）**：

| 用例 | 断言 |
|------|------|
| 偏离超阈 | 消费 3,150→4,200（+33.3%）→ deviations 命中该桶，方向 up |
| 偏离未超 | 备用 30,000→32,000（+6.7%）→ 空 |
| 前值 = 0 | 投资 0→2,000 → 该桶不参与（RULE-025） |
| 负值 | 消费 -1,200 → 参与展示、不参与百分比比对；-3,150→-1,200 不报百分比 |
| 基准回溯 | 上月 special → 基准落上上月；全部 special / 无历史 → deviations = null |
| 同名桶 | 桶结构变化后的首条 → 无偏离；恢复同名 → 恢复比对 |
| 缺口月数 | 目标 77,000、余额 32,000、必要 14,000 → 3.2；余额 ≥ 目标 → met + 0 |
| 已坚持月数 | 1/2/4 月有快照（3 月跳过）→ 3 |
| CSV 转义 | 含引号/逗号/`=` 前缀字段 → 转义正确、无公式注入 |

**service 集成测试**：PUT 同月两连发 → 仍 1 行 + updated_at 推进；DELETE 非最新月 → 422；埋点失败不阻断（mock 注入）；新端点无 Cookie ×5 → 401。

**前端手工清单**：AC-1～AC-16 走查（375px + 桌面 × 亮/暗）；五态对照 prd-e §8.3；Token 门禁（`check-tokens.sh`）；grep 产品名零命中。

**设计闸门**：P05 按 design-e.md 过 T0/T1/T2；T1 自动项（token 逐值、自包含、tabular、焦点、44px、reduced-motion）结果记 test-report-e。

**备份恢复**：restore-drill 增项复跑，记录进 test-report-e。

---

## 11. E 期交付物清单与验收顺序

1. `server/`：迁移 `0007_snapshots` + `domain/tracking.rs`（金例单测）+ `services/snapshot_service.rs` + `api/v1/snapshots.rs` ×5 端点 + 埋点扩展 + ApiDoc 存量补挂 + `.env.example` 增键
2. `web/`：P05 新页（生成产物落地）+ P01 追踪卡 + api-types 重生成
3. `docs/design/`：P05 固化快照 + `生成记录.md` 追加行（design-e §6）
4. `docs/e/`：test-report-e.md（另批）；`.scratch/e-tracking/` 工单（另批，Q5 决策）
5. 验收顺序：域单测绿 → 鉴权链路（curl 无 Cookie 401）→ T1 草稿验收 → 落地 + T2 → 端到端走查（录入 → 偏离 → 导出）→ 埋点 SQL 查验 → restore-drill 增项 → 苑问真实数字打卡（DoD 验收动作）

**存量挂账核对**：RISK-E-1（TLS，接受，§10.2 prd-e）、I-8（收入不足文案歧义，不属本期域，继续挂账）、I-9（prod 恢复演练，挂账）、I-12（非 Chromium 走查，挂账）。
