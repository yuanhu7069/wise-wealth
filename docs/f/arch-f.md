# 架构实例：智策理财 F 期 · 模式库补齐与模式库页（arch-f.md）

> 关联规范：docs/base_line/基线-arch-Web域.md v0.2 / 基线-design-OpenDesign-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260925-01 智策理财 F 期模式库补齐与模式库页（prd-f.md）
> 前序实例：docs/e/arch-e.md（继承 b/arch-v2.md；**增量式**：A/B/E 期已锁定内容不重述，仅记 F 期新增与变更）
> 数据库：外部 PostgreSQL dev 库（连接参数见 `.env`；无 TLS —— RISK-E-1 挂账延续，对外前必须解决）
> 上游总纲：docs/智策理财_PRD_V1.1.md（§4.1 模式库、§7.1 模式核证表）；领域词汇：根 CONTEXT.md「模式体系」节（本期新增「出处」「手动选择」）

---

## 0. 实例 ADR（F 期项目级决策）

> ADR-A-001～004、ADR-B-001～005、ADR-E-001～004 继续有效；本期无推翻。

### ADR-F-001：`source` 进入模式配置 schema——一次性向后兼容扩展，红线边界澄清

【状态】已接受（2026-09-25，grilling Q7 拍板）
【上下文】模式出处目前只写在 TOML 顶部注释（four_accounts.toml:3-5），P06 详情要展示出处就必须结构化。RULE-036 红线是「新增模式不改引擎」；给 `ModeConfig` 加可选字段是否破线需要明确边界。
【决策】`mode.rs` 的 `ModeConfig` 增加可选字段 `source: Option<String>`，并在加载校验中要求**全部模式声明非空 source**（缺失或为空 → 启动失败 fail-fast，同既有坏配置立场）；存量 2 个 TOML 把注释中的出处迁入字段。红线边界据此澄清：**「新增模式零代码」指新增一个模式 TOML 不需要触碰任何 `mode.rs`/`engine.rs` 代码——schema 的一次性扩展是建库动作，不是加模式动作**；今后新增模式仍零代码。
【后果】正面：出处成为一等元数据，P06/知识库（G 期）都有数据可用；fail-fast 保证没有匿名模式。负面：本票触碰 `mode.rs`（一次性成本，记入票 01）；若未来出处需要结构化考据（多来源/链接），字段升级为 JSONB 届时再评估（G 期知识库范围）。

### ADR-F-002：方案的可信度/出处按**读取时**解析，不冻结进快照——本期零迁移

【状态】已接受（2026-09-25）
【上下文】方案页提示条（RULE-035）需要方案模式的 credibility/source。两个候选：冻结进 `plans` 表（迁移 + 回填）或读取时从模式库解析。`plans` 表现状：`l1_mode` 只存 id，`l1_mode_name` 本就是读取时由 ModeLibrary 解析（0004_plans.up.sql 无名称列）——评级与出处跟名称同性质，都是「模式的属性」而非「单次生成的属性」。桶名/金额/比例仍按快照冻结（B 期既定），不受此影响。
【决策】方案响应 DTO 增加 `l1_credibility` / `l1_source`，读取时由 ModeLibrary 按 `l1_mode` id 解析；**零迁移、零回填**。若某模式被下架（TOML 删除），历史方案解析不到 → 字段返回 null，前端不渲染提示条（降级安全）。
【后果】正面：零迁移；模式被重新考证后（disputed → verified）历史方案页的提示随之更新——提示条反映的是模式的**当前**知识状态，语义正确。负面：模式库配置变更会改变历史方案页的提示呈现（桶金额不受影响，可接受并在 ADR 记录）。

### ADR-F-003：理财金字塔推后——需要引擎语义扩展，违反 RULE-036 即不做降级

【状态】已接受（2026-09-25，grilling Q4 拍板）
【上下文】产品 PRD Phase 1 列 5 个 L1 模式；金字塔的「层级优先、逐层上行」需要 priority/层级字段与可配置让位序——现有 `ShareType` 仅 4 种、让位链硬编码（engine.rs safety_first）。降级近似（保障层=fixed 桶 + 保命层=rule 桶 + 增值层=remainder）与四账户同构，金字塔灵魂（层级让位）丢失。
【决策】本期不做金字塔（prd-f OUT-003）；未来若做，前置条件 = ADR 扩引擎获批（新增 ShareType 变体或 priority 字段 + engine 求解序改动 + 金例测试），按「内核扩展」立项而非「加模式」。
【后果】正面：红线零破例；避免交付一个无独立价值的复制品。负面：Phase 1 的「5 个 L1 模式」在本期达成 4/5（four_accounts / fifty_30_20 / snp_quadrant / four_pots），金字塔挂账至引擎扩展获批。

### ADR-F-004：P06 为只读 + 单动作页——列表一次取全，share 口径服务端字符串化

【状态】已接受（2026-09-25，承 ADR-B-004 前端零计算立场）
【上下文】P06 的数据是静态模式库（进程内、启动期已校验），交互仅「展开详情」与「CTA 生成」两个动作。
【决策】① `GET /modes` 响应一次带全卡片 + 详情数据（含 `source` 与桶概览），**详情展开零额外请求**；② 桶份额的人类可读口径（`share_desc`，如「每月收入的 30%」「按应急金节奏划入」）由服务端从 share 类型字符串化，前端不解读 share 结构；③ CTA 复用既有生成端点，请求体新增可选 `entry` 枚举（`questionnaire` | `mode_lib`，缺省 questionnaire）仅用于埋点口径。
【后果】正面：前端零业务计算；P06 无独立数据流，测试面小。负面：列表响应体变大（5 模式 × 桶概览，< 10KB，可接受）。

---

## 1. 页面与路由映射（F 期增量）

| 页面ID | 页面名称 | Next 路由 | 组件类型 | 备注 |
|--------|---------|-----------|---------|------|
| P06 | 模式库页（新） | `/modes` | Server 壳（读模式列表）+ Client 卡片（展开/CTA 交互，Server Action 提交生成） | 生成走 design-f 三道闸门；无表单无财务数字 |
| P01 | 产品首页（改造） | `/` | Server | 导航区追加「模式库」入口；**代码级改动，不回炉生成** |
| P04 | 方案页（改造） | `/plan` | Server + Client | ①「查看全部模式」链接 ② 可信度提示条（读取解析字段） |
| P02/P03/P05 | - | - | - | 零改动（P03 步 6 徽章渲染已满足 RULE-033） |

鉴权边界：`/modes` 进 B 期 layout 层未登录重定向链（`/login?from=/modes`）；服务端强制仍在 Rust 中间件（`GET /modes` 已有 JWT，白名单**零变更**）。

## 2. 核心实体与数据模型（F 期增量）

**零数据库迁移**——本期显著事实：模式为配置内嵌（build.rs 扫描 + include_str），方案响应扩展走读取解析（ADR-F-002），`snapshots`/`plans` 表结构零变更。

`ModeConfig` schema 扩展（mode.rs，ADR-F-001）：

| 字段 | 类型 | 说明 |
|------|------|------|
| `source` | `Option<String>`（加载校验**非空**） | 出处说明；disputed 模式必须含辟谣表述（RULE-033 的数据面） |

### 2.1 新增模式 TOML 规格（配置即数据）

**snp_quadrant（标准普尔象限 · disputed · fifty_30_20 同形状族：纯 pct 恰 10000，无余量/规则桶）**

| 桶 id | 名称 | share | 标记 | 用途 |
|-------|------|-------|------|------|
| spending | 要花的钱 | pct 1000（10%） | is_necessary | 日常开销 |
| protection | 保命的钱 | pct 2000（20%） | - | 保障与保险 |
| growth | 生钱的钱 | pct 3000（30%） | is_investable | 投资增值 |
| preserve | 保本的钱 | pct 4000（40%） | - | 保本升值 |

`source` = 网传「标准普尔家庭资产象限图」；标准普尔公司从未发布，系 2011 年前后国内保险业自创（总 PRD §7.1 考据）。`emergency_fund` 档位沿用既有模式同族取值（拷贝 fifty_30_20 配置，票 01 落地时核对）。语义取舍（Q6 拍板）：忠实原典静态比例——单投资桶（生钱的钱）、无让位语义；保命的钱不做规则桶（按 pct 静态划入）。

**four_pots（四笔钱 · verified · four_accounts 同形状族：fixed + rule + pct + remainder）**

| 桶 id | 名称 | share | 标记 | 用途 |
|-------|------|-------|------|------|
| liquid | 活钱 | fixed_expenses | is_necessary | 日常开销扣划（要花的钱） |
| protection | 保障 | rule(emergency_fund) | - | 应急金与保障储备（保命的钱） |
| stable | 稳钱 | pct 2000（20%） | - | 稳健理财（波动小的钱） |
| long_term | 长钱 | remainder | is_investable | 长期投资（长钱） |

`source` = 盈米且慢「四笔钱」资金规划框架（国内产品化；按资金久期分流：活钱管理 / 保险保障 / 稳健理财 / 长期投资）。

**口径决定（票 01 落地时记入 Comments）**：① 应急金按引擎规则桶语义归**保障桶**（「保命的钱」语义涵盖应急与保障），追踪模块的应急桶回落链（E 期 RULE-026：规则桶优先）自动命中；② 稳钱 20% 为配置值（框架本身不定比例，配置即数据可调）；③ 保险保费不入桶（可计入日常固定支出由活钱承载），P06 用途文案如实描述。

加载期校验（既有机制，零新增代码）：snp_quadrant 过「无余量桶 → pct 恰 10000」；four_pots 过「有规则桶 → 必有余量桶 + pct < 10000」；source 非空校验为**唯一新增校验**（ADR-F-001）。

## 3. 需求 → 能力映射（RULE-032 起 → 代码落点）

| RULE | 后端实现位置 | 前端实现位置 |
|------|-------------|-------------|
| RULE-032（P06 展示口径） | `api/v1/modes.rs`（DTO 扩 source + 桶概览 + share_desc，ADR-F-004） | `web/src/app/modes/`（卡片/详情/徽章/CTA 渲染） |
| RULE-033（disputed 不主推 + 辟谣） | `services/recommend_service.rs` **零改动**（RULE-006 两分支天然不含新模式，测试锁定 disputed 永不 is_recommended）+ TOML source 辟谣文案 | P06 徽章与辟谣块；P03 步 6 徽章（既有） |
| RULE-034（手动生成） | `api/v1/plans.rs`（entry 可选参数 + 枚举校验） | P06 CTA（Server Action 复用 `generatePlanAction` 链路） |
| RULE-035（方案页提示条） | `dto/plan`（l1_credibility / l1_source 读取解析，ADR-F-002） | P04 提示条三态映射（disputed=warn / caution=info / verified=无） |
| RULE-036（配置红线） | `mode.rs` / `engine.rs` **零改动**；红线由本表 + 评审清单 + 金例测试承载 | - |
| RULE-037（source 必备） | `mode.rs`（字段 + 加载校验）+ 全部 5 个 TOML | - |
| 埋点（§9.5） | `services/analytics_service.rs`：`plan_generated` payload 增 `entry`；PageId 枚举 +P06 | `page_view` p06（既有上报组件，白名单扩） |

## 4. API 清单（F 期变更，均挂 `/api/v1`）

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| GET | `/modes` | 既有端点，响应扩展 | JWT | - | items[] 每项增 `source?: string`、`buckets: [{name, purpose, share_desc}]`（ADR-F-004） |
| POST | `/plans` | 既有端点，请求体扩展 | JWT | `{l1_mode, entry?: "questionnaire"\|"mode_lib"}`（缺省 questionnaire；未知枚举 422） | 不变 |
| GET | `/plans/active` | 既有端点，响应扩展 | JWT | - | 增 `l1_credibility?: "verified"\|"disputed"\|"caution"`、`l1_source?: string`（读取解析，模式下架 → null） |
| POST | `/analytics/events` | 既有端点，白名单扩 | JWT | `page_view` + page_id=p06 | 不变 |

代码gen链路不变：DTO 变更后重跑 `scripts/gen-types.sh`（`api-types.ts` 禁手改）。

## 5. 权限校验点清单（F 期增量）

| 校验点 | 位置 | 校验内容 | 越权/未认证时行为 |
|--------|------|---------|------------------|
| 中间件全路由校验 | `api/middleware/auth.rs` | 全部涉及端点均为**既有**端点（白名单零变更） | 401 UNAUTHORIZED 信封 |
| entry 枚举 | `dto/plan_dto.rs` | questionnaire / mode_lib 之外 422 | 422 VALIDATION_ERROR |
| 模式存在性 | `plan_service.rs`（既有） | 未知 l1_mode 422 | 422（既有） |
| 页面鉴权 | Next layout 层 | `/modes` 进未登录重定向链 | 302 → `/login?from=/modes` |
| 埋点白名单 | `analytics_service.rs` | PageId 枚举 +P06 | 422 |

越权/降级测试：`GET /modes`、`POST /plans` 无 Cookie → 401；`entry="elsewhere"` → 422；PG 停机 → P06 错误态（AC-11）。

## 6. 外部依赖与集成（F 期增量）

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| Open Design daemon 0.22.1（127.0.0.1:7456） | P06 界面生成（**生成期一次性**，运行时零依赖） | 连不上 → 停下报告苑问，不手搓、不换来源（基线 §2.2） | BYOK（用户侧） |
| 其余 | **无新增 crate / 中间件 / 环境变量键** | - | - |

`.env.example` 零新增；无新脚本。

## 7. 数据与迁移（F 期增量）

- **零迁移**：无新表、无列变更（ADR-F-002 是零迁移的前提，两 ADR 绑定评审）
- 备份与恢复：`restore-drill.sh` 复跑一次确认既有抽验清单全绿（表结构零变更故**无增项**），记录回填 test-report-f；B 期 I-9 的 prod 分支演练继续挂账
- 红线 15 自查：本期不触碰资金类数据结构 ✅

## 8. 安全要点（F 期增量）

- 敏感面：P06 无财务数字（桶概览为比例与口径描述）；`source`/`tagline` 为静态配置文本，无注入面（不出现在 CSV/公式语境）
- 输入面：entry 枚举校验；l1_mode 存在性校验（既有）
- 输出面：模式可信度与出处仅登录后可见（随既有鉴权链）；金额不入日志与埋点立场继承
- codegen：`api-types.ts` 重生成后零手改核对

## 9. 部署与启动（增量）

拓扑不变（WSL2 双服务 → 外部 PG）。`start.sh` 不改（**无迁移**）；`check.sh` 不改（新增用例自动纳入 cargo test）。新增 TOML 改变 `config/modes` 目录 mtime → 下次构建自动拾取（build.rs rerun-if-changed），部署动作 = 重新构建启动，无额外步骤。

## 10. 测试计划（基线 §14.3 模式，承接 B/E 期）

**域/加载层单测（`cargo test`，金例锁定）**：

| 用例 | 断言 |
|------|------|
| snp_quadrant 求解 | 收入 12,400 → 四桶 1,240 / 2,480 / 3,720 / 4,960；L2 作用于 growth 桶 |
| snp_quadrant 合法性 | 无余量桶 + pct 恰 10000 通过加载校验 |
| four_pots 求解 | fixed（月固定支出）+ rule（应急节奏）+ 20% + 余量；L2 作用于 long_term 桶 |
| four_pots 合法性 | 有规则桶 + 有余量桶 + pct < 10000 通过加载校验 |
| source fail-fast | 任一 TOML 缺 source / source 为空 → 加载失败退出非 0（RULE-037） |
| share_desc | 四种 share 类型字符串化正确（含 fixed/pct/rule/remainder） |
| entry 校验 | `entry="elsewhere"` → 422；缺省 → questionnaire |
| credibility 解析 | plans 响应按读取时库解析；未知模式 id → null（ADR-F-002 降级） |
| disputed 不主推 | 任意档案输入下 `is_recommended` 永不为 disputed 模式（RULE-033 锁定） |

**service/API 集成测试**：新 TOML 上线后 `GET /modes` 含 5 项且字段齐；`POST /plans` entry=mode_lib 落 `plan_generated` 埋点带 entry；无 Cookie ×2 → 401。

**前端手工清单**：AC-1～AC-13 走查（375px + 桌面 × 亮/暗）；五态对照 prd-f §8.3；Token 门禁（`check-tokens.sh`）；grep 产品名零命中（P06 出处文案不含具体金融产品名）。

**设计闸门**：P06 按 design-f.md 过 T0/T1/T2；T1 自动项结果记 test-report-f。

**备份恢复**：restore-drill 复跑（无增项），记录进 test-report-f。

---

## 11. F 期交付物清单与验收顺序

1. `server/`：mode.rs source 字段 + 加载校验 + 2 个新 TOML + 存量 2 个 TOML 补 source + modes/plans DTO 扩展 + entry 参数 + 埋点扩展（**零迁移、零 engine 改动**）
2. `web/`：P06 新页（生成产物落地）+ P01 入口 + P04 链接与提示条 + api-types 重生成
3. `docs/design/`：P06 固化快照 + `生成记录.md` 追加行（design-f §6）
4. `docs/f/`：test-report-f.md（另批）；`.scratch/f-modes/` 工单（同批产出）
5. 验收顺序：TOML 加载与金例绿 → 鉴权链路（curl 无 Cookie 401）→ T0 自查 → 生成 → T1 草稿验收 → 固化 → 落地 + T2 → 端到端走查（浏览 → 详情 → CTA 生成 → 提示条）→ 埋点 SQL 查验 → restore-drill 复跑 → 苑问真实生成（DoD 验收动作：四笔钱生成 + disputed 链路走查）

**存量挂账核对**：RISK-E-1（TLS，挂账延续）、I-8（收入不足文案歧义，挂账）、I-9（prod 恢复演练，挂账）、I-12（非 Chromium 走查，挂账）、**理财金字塔（新挂账，OUT-003，前置 = 引擎扩展 ADR）**。
