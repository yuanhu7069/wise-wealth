# 架构实例：智策理财 G 期 · 知识库 V1 与模式对比（arch-g.md）

> 关联规范：docs/base_line/基线-arch-Web域.md v0.2 / 基线-design-OpenDesign-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260925-02 智策理财 G 期知识库 V1 与模式对比（prd-g.md）
> 前序实例：docs/f/arch-f.md（继承 b/arch-v2.md、e/arch-e.md；**增量式**：已锁定内容不重述，仅记 G 期新增与变更）
> 数据库：外部 PostgreSQL dev 库（连接参数见 `.env`；无 TLS —— RISK-E-1 挂账延续）
> 上游总纲：docs/智策理财_PRD_V1.1.md（§4.5 知识库、§4.1.3 对比）；领域词汇：根 CONTEXT.md「知识库」节（本期新增五术语）

---

## 0. 实例 ADR（G 期项目级决策）

> ADR-A/B/E/F 各期决策继续有效；本期无推翻。

### ADR-G-001：知识内容 = `server/config/knowledge/` 结构化 TOML，构建期内嵌——内容即配置

【状态】已接受（2026-09-25，grilling Q3 拍板）
【上下文】知识库是长文集合（16 篇起）。三个候选真源：数据库（可后台编辑但本期无后台、无版本史优势）、前端 MDX（markdown 写作体验好但必须引渲染依赖，破红线 1）、服务端配置内嵌（与模式库同机制）。
【决策】新建 `server/config/knowledge/*.toml`（扁平目录，篇一文件），`build.rs` 扩展扫描并 `include_str!` 内嵌；新建 `domain/knowledge.rs` 定义 `KnowledgeArticle { id, kind, title, related_mode: Option<String>, summary, sections: Vec<Section{heading, paragraphs}> }`，kind 枚举 `mode_interpretation | verification | encyclopedia | non_implementable`；装载期校验：id 全库唯一、sections 非空、解读类 related_mode 必须存在于模式库且 sections 必含 heading「局限性」、不可落地类必含 heading「为什么不建议照搬」；坏配置启动失败（同模式库 fail-fast）。前端按 sections 结构渲染，**不引入 markdown 渲染器**。
【后果】正面：加内容零代码（同「加模式零代码」承诺）；进 Git 有版本史；零迁移零依赖；校验兜底内容结构完整性。负面：长文写作比 markdown 啰嗦（结构换可控排版，RISK-G-4 记录，写作体验确实差则 Phase 2 再评估）；正文含 `#` 等字符无需转义（TOML 字符串），但换行需多行字符串（`'''`）。

### ADR-G-002：试算端点 `POST /api/v1/plans/preview`——只读复用 solve，显式例外于 REST 语义

【状态】已接受（2026-09-25，grilling Q2 拍板）
【上下文】对比表格的金额列需要「以当前档案对指定模式跑引擎」。用 GET 传模式列表语义别扭，用 POST 语义上是「计算请求」而非「创建资源」——与统一信封同存的 REST 惯例需要一次显式取舍（同 ADR-E-004 的导出例外立场）。
【决策】`POST /api/v1/plans/preview`，请求体 `{mode_ids: [1..=3]}`；服务端读当前档案（不完整 → 422）后对每个 id 调 `engine::solve`——**不写库、不产生版本、不改 active、不影响推荐**；响应逐模式返回 `{mode_id, ok, solution|reason}`（solution 内含 buckets 金额、L2、应急状态、notices；不可行模式 ok=false + reason），并内嵌模式元数据（name/credibility/source/tagline/fit_for/桶 share_desc）使对比页**单请求渲染**。非法 id / 数量超界 → 422。
【后果】正面：与生成共用同一纯函数，对账一致性有结构保证（AC-6）；P08 无二次请求。负面：POST 用于计算是记录在案的显式例外；将来 Plus 的参数化试演将复用本端点（加档案覆盖参数，届时扩展）。

### ADR-G-003：P07/P08 双页生成 + P06 代码级增量；对比勾选态走 URL

【状态】已接受（2026-09-25，grilling Q4 拍板）
【上下文】知识库与对比是两个独立信息架构（内容子站 vs 交互工具）；P06 上期刚过验收。勾选态需要在「勾选 → 对比 → 返回」链路中存活，且最好可分享。
【决策】`P07 /knowledge` 与 `P08 /modes/compare` 走 Open Design 生成（各 1 run）；P06 增量（勾选交互 + 「开始对比」+ 「阅读完整解读」链接）为**代码级改动**（基线：落地后真源 = 代码库，同 E 期 P01 卡先例）。P08 勾选态以 URL 查询参数承载（`?modes=a,b,c`），服务端渲染即从 URL 取数，刷新/回跳/分享不丢状态。
【后果】正面：生成预算 2 run 边界清晰；URL 态零持久化成本。负面：P08 的可分享性在单用户系统里是顺带收益而非需求（不额外做短链）。

### ADR-G-004：`api-types.ts` 手工移植延续，gen-types.sh 修复升格为期收口拍板项

【状态】已接受（2026-09-25；承 F 期票 02 偏离记录）
【上下文】gen-types.sh 因 openapi-typescript 与 TS7 不兼容不可用（E 期挂账），F 期已按「真实 openapi.json 为准手工移植」先例执行一次。
【决策】本期延续手工移植（新增 KnowledgeView / PreviewView 等类型），移植时以 `curl /api/v1/openapi.json` 实际输出为准、逐字段对照；每处移植在工单 Comments 记录。**gen-types.sh 修法（升级 openapi-typescript / 降级 TS / 锁 Node）列入本期收口拍板项**，连续第三期手工移植不可接受。
【后果】正面：不阻塞本期。负面：手工移植有人为漏错风险（F 期已发生一次缩进漏移植，靠 tsc 兜住）；拍板前每次 DTO 变更都要多一双眼睛。

---

## 1. 页面与路由映射（G 期增量）

| 页面ID | 页面名称 | Next 路由 | 组件类型 | 备注 |
|--------|---------|-----------|---------|------|
| P07 | 知识库页（新） | `/knowledge`；文章阅读态 `/knowledge?id=<id>`（同页查询参数展开） | Server 壳（取知识列表）+ 文章区（server 渲染 sections） | 四板块分区；生成走 design-g 三道闸门 |
| P08 | 对比页（新） | `/modes/compare?modes=a,b,c` | Server 壳（解析 URL + 调 preview）+ 表格（server 渲染） | 零客户端状态；单请求渲染（ADR-G-002） |
| P06 | 模式库页（改造） | `/modes` | + Client 勾选状态 | 每卡「加入对比」+「开始对比」+「阅读完整解读」；**代码级，不回炉** |
| P01-P05 | - | - | - | 零改动 |

鉴权边界：`/knowledge`、`/modes/compare` 进 requireSession 重定向链（`/login?from=<path>`）；服务端强制仍在 Rust 中间件（新端点全部落 JWT，白名单**零变更**）。P06 的既有登录前置同时保证档案完整（GET /modes 422 兜底）。

## 2. 核心实体与数据模型（G 期增量）

**零数据库迁移**（连续第二期）：知识内容内嵌二进制；试算不落库；plans/snapshots 零变更。

### 2.1 知识内容 schema（ADR-G-001）

| 字段 | 类型 | 校验 |
|------|------|------|
| `id` | 字符串，全库唯一 | 重复 → 启动失败 |
| `kind` | 枚举四值 | `mode_interpretation` / `verification` / `encyclopedia` / `non_implementable` |
| `title` | 非空 | - |
| `related_mode` | 可选 | mode_interpretation 必填且必须存在于模式库；其余禁止携带 |
| `summary` | 非空一句话 | 卡片导语 |
| `sections` | `[{heading, paragraphs[]}]` 非空 | 解读类必含 heading「局限性」；不可落地类必含「为什么不建议照搬」 |

### 2.2 本期内容清单（16 篇，票 01 起草、苑问审读）

| kind | 篇目 |
|------|------|
| mode_interpretation ×4 | 四账户理财法 / 50/30/20 / 四笔钱 / 标准普尔象限（related_mode 一一对应；每篇四节：理论来源 / 核心逻辑 / 适用条件 / **局限性**） |
| verification ×1 | 标准普尔象限考据（流传史 / 证伪要点 / 与 verified 模式的差异） |
| encyclopedia ×8 | 再平衡 / 费率 / 应急金 / 复利 / 通胀 / 资产大类 / 定投 / 回撤（每篇 2-3 节，含「与本产品的关系」） |
| non_implementable ×3 | 生命周期杠杆版 / 美林时钟 / 耶鲁模式（每篇含「为什么不建议照搬」） |

## 3. 需求 → 能力映射（RULE-038 起 → 代码落点）

| RULE | 后端实现位置 | 前端实现位置 |
|------|-------------|-------------|
| RULE-038（内容即配置 + fail-fast） | `domain/knowledge.rs`（schema + 校验）+ `build.rs` 扫描扩展 | P07 按结构渲染 sections |
| RULE-039（解读关联模式 + 局限性必写） | `knowledge.rs` 装载校验 | 解读卡徽章来自 GET /modes 元数据 |
| RULE-040（不可落地标注 + 引擎分离） | 校验「为什么不建议照搬」；knowledge 条目**不进**模式库/preview 的合法 id 集 | P07 专区警示徽章（渲染层统一） |
| RULE-041（对比 ≤3 参数表格） | preview 端点 mode_ids 数量校验 | P06 勾选上限 + P08 表格 |
| RULE-042（试算只读不落库） | `api/v1/plans.rs::preview`（solve 复用，零写库调用） | P08 金额列 |
| RULE-043（如实提示不编金额） | preview 响应 ok=false + reason | P08 不可行列警示短语 |
| RULE-044（内容合规） | 内容审读（人工环节）+ TOML 为唯一文本源 | P07/P08 页脚免责（SiteFooterShell 复用） |
| 埋点（§9.5） | PageId 枚举 +P07/P08 | page_view p07/p08（白名单扩） |

顺手修复：无（F 期 ApiDoc 补挂后链路完整；本期新端点随建随挂）。

## 4. API 清单（G 期新增，均挂 `/api/v1`）

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| GET | `/knowledge` | 文章元数据列表（id/kind/title/related_mode/summary，**不含 sections**） | JWT | - | `{items[]}` 按 kind 分组序 |
| GET | `/knowledge/:id` | 文章全文（含 sections） | JWT | - | `{article}`；未知 id → 404 |
| POST | `/plans/preview` | 试算（只读，ADR-G-002） | JWT | `{mode_ids: [1..=3]}` | `{items: [{mode_id, ok, meta{...}, solution?{buckets[], l2_name, emergency, notices}, reason?}]}`；档案不完整/非法 id → 422 |
| POST | `/analytics/events` | 既有端点，白名单扩 | JWT | page_view + p07/p08 | 不变 |

P07 阅读态用列表 + 详情两次取数（列表进各板块卡，详情随文章展开按需取）；P08 单请求（preview 内嵌元数据）。codegen：DTO 变更后手工移植 api-types（ADR-G-004）。

## 5. 权限校验点清单（G 期增量）

| 校验点 | 位置 | 校验内容 | 越权/未认证时行为 |
|--------|------|---------|------------------|
| 中间件全路由校验 | `api/middleware/auth.rs` | 3 个新端点全部落 JWT（白名单零变更） | 401 UNAUTHORIZED 信封 |
| mode_ids 数量与存在性 | `dto/plan_dto.rs`（preview） | 1..=3；仅限模式库内 L1 id（knowledge 条目天然不在） | 422 VALIDATION_ERROR |
| 档案完整性 | `api/v1/plans.rs::preview` | 未答完问卷 → 422（与生成同口径） | 422 |
| 文章 id 存在性 | `api/v1/knowledge.rs` | 未知 id → 404 信封 | 404 NOT_FOUND |
| 页面鉴权 | requireSession | /knowledge、/modes/compare 进重定向链 | 302 → /login?from= |
| 埋点白名单 | `analytics_service.rs` | PageId 枚举 +P07/P08 | 422 |

越权/降级测试：3 新端点无 Cookie → 401；`mode_ids=4 个` → 422；未知文章 id → 404。

## 6. 外部依赖与集成（G 期增量）

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| Open Design daemon 0.22.1（127.0.0.1:7456） | P07/P08 界面生成（**生成期一次性 ×2 run**，运行时零依赖） | 连不上 → 停下报告苑问，不手搓（基线 §2.2） | BYOK（用户侧） |
| 其余 | **无新增 crate / 中间件 / 环境变量键** | - | - |

## 7. 数据与迁移（G 期增量）

- **零迁移**（连续第二期）；restore-drill 复跑无增项，记录回填 test-report-g
- 内容文件进 Git（`server/config/knowledge/`），修订 = 改 TOML 重新构建
- 红线 15 自查：不触碰资金类数据 ✅

## 8. 安全要点（G 期增量）

- 内容文本为构建期内嵌静态数据，无用户输入拼接 → 无注入面；不进 CSV/公式语境
- 试算金额仅登录后可见；**金额、试算结果不入日志与埋点**（tracing 只带 request_id + 路由名）
- preview 无写库路径（代码评审清单项：preview 函数内禁止出现任何 `*_repo::write/insert/update`）
- 输入面：mode_ids 数量/存在性、文章 id 存在性全量 DTO 校验

## 9. 部署与启动（增量）

拓扑不变。`start.sh` 不改（零迁移）；`check.sh` 不改（新增用例自动纳入）。新增 knowledge 目录改变 mtime → 构建自动拾取（build.rs rerun-if-changed 同模式库机制）。

## 10. 测试计划（基线 §14.3 模式）

**域/加载层单测（金例锁定）**：

| 用例 | 断言 |
|------|------|
| 内嵌知识装载 | 16 篇齐、kind 分组计数 4/1/8/3、id 唯一 |
| 解读校验 | related_mode 指向不存在模式 → 启动失败；缺「局限性」节 → 启动失败 |
| 不可落地校验 | 缺「为什么不建议照搬」节 → 启动失败 |
| id 重复 | 两篇同 id → 启动失败 |
| preview 一致性 | 同档案同模式下 preview.solution == 生成路径 solve 结果（**结构对账单测**，AC-6 的自动化底座） |
| preview 边界 | mode_ids 空/超 3/含未知 id → 422；不可行模式 ok=false + reason（金例：收入 800/固定 700 档案） |
| knowledge 端点 | 列表不含 sections；未知 id → 404；无 Cookie ×3 → 401 |

**前端手工清单**：AC-1～14 走查（375px + 桌面 × 亮/暗）；五态对照 prd-g §8.3；Token 门禁；grep 产品名零命中（16 篇内容逐一 grep）。

**设计闸门**：P07/P08 各按 design-g.md 过 T0/T1/T2；T1 自动项记 test-report-g。

**备份恢复**：restore-drill 复跑（无增项）。

## 11. G 期交付物清单与验收顺序

1. `server/`：build.rs 扫描扩展 + `domain/knowledge.rs` + 16 篇内容 TOML + `api/v1/knowledge.rs` ×2 端点 + preview 端点 + PageId +P07/P08（**零迁移、零引擎改动**）
2. `web/`：P07 新页（生成落地）+ P08 新页（生成落地）+ P06 增量（勾选/链接）+ api-types 手工移植
3. `docs/design/`：P07/P08 固化快照 + `生成记录.md` 追加两行
4. `docs/g/`：test-report-g.md（另批）；`.scratch/g-knowledge/` 工单（同批）
5. 验收顺序：内容 TOML 装载与金例绿 → preview 对账单测绿 → 鉴权链路 → T0 自查 → P07 生成 → T1 → 固化 → P07 落地 + T2 → P08 生成 → T1 → 固化 → P08 落地 + T2 → P06 增量 → 端到端走查（勾选 → 对比 → 对账 → 深读）→ 埋点 SQL → restore-drill → 苑问内容审读 + DoD 验收动作

**存量挂账核对**：RISK-E-1（TLS）/ I-8 / I-9 / I-12 / 金字塔（OUT-003）/ gen-types.sh TS7（升格拍板项，ADR-G-004）/ js-yaml audit fix / dev hydration 环境问题 / AC-10 caution 缺口——全部延续，无新增除上述外。
