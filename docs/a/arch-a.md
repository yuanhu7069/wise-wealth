# 架构实例：智策理财 · A 期工程骨架（arch-a.md）

> 关联规范：基线-arch-Web域.md v0.2 / 基线-design-Web域.md v0.1 / 基线-PRD-Web域.md v0.2
> 对应需求：REQ-20260907-01 智策理财 · A 期工程骨架（prd-a.md）
> 数据库：外部 PostgreSQL 公网云实例（不在本机），A 期使用 dev 库（库名见 .env）；实例连接参数见 `.env`，**不写入本文件**
> 上游总纲：docs/智策理财_PRD_V1.1.md（产品路线图 Phase 1-4）

---

## 0. 实例 ADR（项目级决策）

> 基线 ADR-001 ~ 008 继续有效，此处只记录**项目特有**决策。

### ADR-A-001：A 期不做鉴权，B 期第一个业务端点前必须引入

【状态】已接受（2026-09-07）
【上下文】A 期仅 `/health` 与工程验证页，无业务数据、无用户输入；部署面为开发机网络。基线 §4.7 第 6 条要求「单用户也必须鉴权」。
【决策】A 期不引入 JWT / argon2 / 登录页。**放行条件**：`/health` 响应字段白名单化（RULE-001），页面不含业务数据。B 期以「首个业务端点落地」为触发点，同批交付鉴权（JWT HttpOnly Cookie + argon2 + 越权测试），届时修订本 ADR 状态。
【后果】
- 正面：A 期骨架聚焦工程链路验证，不提前背账号体系的复杂度
- 负面：`/health` 在局域网内可被任意访问（仅泄露版本号，可接受）；B 期工作量前置承诺（已记入 PRD OUT-001）

### ADR-A-002：备份策略自首张业务表落地起强制

【状态】已接受（2026-09-07）
【上下文】基线红线 15：涉及资金/资产数据的项目无备份策略不允许上线。A 期无业务表，尚无「资产数据」可备份；数据库实例侧的常规备份由实例运维负责（不在本仓库范围）。
【决策】A 期不做应用级备份脚本；**首个含业务数据的迁移合并时**，必须在同批次交付：`scripts/backup.sh`（pg_dump）+ 恢复验证记录（写入本文件 §7）。智策理财后续必然涉及财务数据，此决策只延迟、不豁免。
【后果】正面：A 期不写空架子脚本；负面：B 期首个业务功能的工作量清单多一项（已在 PRD §5.3 提示）。

### ADR-A-003：/health 恒返回 HTTP 200，用 data.db 表达数据库状态

【状态】已接受（2026-09-07）
【上下文】基线 §11.2 要求 `/health` 返回「服务与数据库连通状态」。两种流派：库挂时返回 5xx（面向负载均衡摘除节点），或恒 200 + 状态字段（面向人读与简单探测）。
【决策】服务进程存活即 200；数据库可用性放 `data.db`（`"ok"` / `"error"`）。理由：A 期无负载均衡，消费方是人与 P01；把「进程活着」与「依赖可用」两个信号分开，避免误判（进程崩溃与依赖故障混在一个 5xx 里）。
【后果】正面：P01 可区分 ERR-001（进程不可达）/ ERR-002（库不可达）两种错误态；负面：若将来上负载均衡探活，需另加严格模式（如 `?strict=1`），届时补 ADR。

### ADR-A-004：A 期不引入 Redis 等缓存

【状态】已接受（2026-09-07）
【上下文】缓存中间件已部署（外部实例）；但 A 期无热数据、无重复计算，基线 §7.1 要求缓存按场景决定并写 ADR。
【决策】不接入。未来引入条件：出现可测量的重复计算（如同一用户短时间重复生成方案）或蒙特卡洛类计算需要结果缓存时，另写 ADR。
【后果】正面：零额外依赖；负面：无（可后补）。

---

## 1. 页面与路由映射

| 页面ID | 页面名称 | Next 路由 | 组件类型（Server/Client） |
|--------|---------|-----------|--------------------------|
| P01 | 工程骨架首页 | `/`（`web/src/app/page.tsx`） | Server（取数）+ 状态卡片子组件 Client（重试交互）；`loading.tsx` 承载骨架屏 |

## 2. 核心实体与数据模型

A 期**无业务实体**。数据库侧仅确认连通性；首张业务表随 B 期建立。

预留约定（B 期起生效，此处仅登记）：

| 约定 | 内容 | 来源 |
|------|------|------|
| 金额 | `BIGINT` 分，字段 `_cents` 后缀，Rust `i64`，禁 float | 基线 ADR-004 |
| 百分比/费率 | 万分比整数 | 基线 §7.4 |
| 时间 | `timestamptz` UTC | 基线 §6.5 |
| ID | UUIDv7 | 基线 §6.5 |
| 软删除 | 业务表默认 `deleted_at` | 基线 §4.3 |

## 3. 需求 → 能力映射

| prd 需求/RULE 编号 | 后端实现位置（crate::模块） | 前端实现位置 |
|-------------------|---------------------------|-------------|
| RULE-001（health 信封语义） | `server/src/api/v1/health.rs` + `server/src/services/health_service.rs` | `web/src/app/page.tsx`（按 `data.db` 渲染） |
| RULE-002（配置校验） | `server/src/config.rs`（启动期 `expect` 式失败，错误列出全部缺失键） | 不适用（终端输出） |
| RULE-003（统一信封） | `server/src/error.rs`（`AppError` + `ErrorCode` + `IntoResponse`） | `web/src/lib/api.ts`（解包 + errorCode 映射） |
| RULE-004（BFF 唯一出口） | -（后端无感知） | `web/src/lib/api.ts`（服务端环境变量 `API_BASE_URL`） |
| RULE-005（类型生成物） | `server/src/openapi.rs`（utoipa 注解） | `web/src/lib/api-types.ts`（生成物，禁止手改） |
| RULE-006（Token 铁律） | - | `web/src/app/globals.css`（`@theme` 全量 Token）；组件零裸值 |
| RULE-007（版本管理） | `.sqlx/`、`Cargo.lock` 提交；`.gitignore` 排除 `target/` | lockfile 提交；排除 `node_modules/`、`.next/` |
| RULE-008（错误三要素文案） | -（错误码只出机器标识） | `web/src/lib/errors.ts`（errorCode → 文案映射表） |
| AC-5（类型变更暴露） | - | `tsc --noEmit` 验证（验收动作） |
| AC-7（暗色模式） | - | `globals.css` 暗色 token 映射（design-a.md §4） |

## 4. API 清单

| 方法 | 路径 | 用途 | 权限 | 请求 | 响应 |
|------|------|------|------|------|------|
| GET | `/api/v1/health` | 服务与数据库连通探测 | 公开（ADR-A-001） | 无 | 信封包裹 `HealthData { status: "ok", db: "ok"\|"error", version: string }` |
| GET | `/api/v1/version` 暂不实现——版本号并入 health，避免 A 期多一个端点 | - | - | - | - |

> 注：基线 §11.2 的 `/health` 与本表 `/api/v1/health` 的关系——对外探测用带版本路径；另在根路径 `GET /health` 提供 302 或同义转发由 §9 启动验证脚本消费。**最终实现取其一**：A 期决定只实现 `GET /api/v1/health`，根路径 `/health` 由 axum 直接挂同 handler（零成本双挂），满足基线字面要求。

错误响应：全部走基线 §6.1 信封；A 期可能出现的 errorCode 仅 `INTERNAL_ERROR`（如 pg pool 初始化失败转发的兜底）。

## 5. 权限校验点清单

| 校验点 | 位置 | 校验内容 | 越权时行为 |
|--------|------|---------|-----------|
| （A 期无） | - | 全端点公开，依据 ADR-A-001 | - |

> B 期模板：每个业务端点登记「校验点 / 中间件位置 / 校验 JWT+资源归属 / 403 FORBIDDEN 信封」。

## 6. 外部依赖与集成

| 依赖 | 用途 | 失败降级 | 密钥管理 |
|------|------|---------|---------|
| 外部 PostgreSQL（库名以 .env 为准，当前暂用 `wise_wealth_db` 作 dev 库） | 连通性验证（A 期唯一外部依赖） | `/health` `db=error`；P01 错误态；**服务进程不退出** | `DATABASE_URL_DEV` 环境变量，禁入日志（prd-a.md §11） |
| crates.io / npm registry | 依赖拉取 | 构建失败（开发期可接受）；lockfile 保证可复现 | 无密钥 |

新增 crate/npm 包审批记录：

| 包 | 用途 | 审批 |
|----|------|------|
| 后端：tokio / axum / sqlx / serde / validator / thiserror / anyhow / tracing / tracing-subscriber / utoipa / dotenvy / uuid | 均在基线 §1.1 清单内 | 基线已批准 |
| 前端：next / react / react-dom / typescript / tailwindcss / @tailwindcss/postcss / shadcn CLI / biome / openapi-typescript / lucide-react | 均在基线 §1.2 清单内 | 基线已批准 |

**清单外依赖一律不引入**（基线红线 1）；如需，先停下来问苑问。

## 7. 数据与迁移

- **初始迁移**：A 期建立 `server/migrations/` 目录并提交 `0001_init_baseline.sql`（仅建 `schema_migrations` 由 sqlx-cli 自管；无业务表——空迁移用于打通 `sqlx migrate run` 链路）。UP/DOWN 均可执行。
- **索引**：暂无（无业务表）。
- **备份策略**：见 ADR-A-002——首张业务表合并时强制补 `scripts/backup.sh` + 恢复演练记录。演练记录：＿（待 B 期回填）
- **`.sqlx/` 离线缓存**：开发机需可连 dev 库（连接串见 `.env`）时执行 `cargo sqlx prepare`，产物提交 Git（基线 ADR-002）。
- **开发库与正式库分离**：RULE-002 强制 `DATABASE_URL_DEV` / `DATABASE_URL_PROD` 指向不同库；A 期只创建并使用 dev 库。

## 8. 安全要点

- **敏感字段**：A 期无业务敏感字段；配置类敏感值（数据库口令）只存在于服务器端 `.env`。
- **脱敏要求**：日志与 `/health` 响应不得出现连接串、主机名、环境变量值；配置启动摘要只输出「已配置/缺失」状态（基线 §8.3）。
- **金额处理**：本期无金额。基线 ADR-004 全链路 i64 分规则自 B 期首个金额字段起强制。
- **响应头**：axum 侧补 `X-Content-Type-Options: nosniff`（中间件一行）；HTTPS/CSP/Cookie 相关自 B 期鉴权引入时按基线 §8.4 落地。
- **依赖审计**：A 期结束时跑一次 `cargo audit` 与 `npm audit`，结果记入 test-report。

## 9. 部署与启动

**部署拓扑（实例层，基线 §0.0 授权此处落地）**：

```
Windows 浏览器（Chrome）
   │  http://<WSL2-IP>:3000
   ▼
Next.js dev server（WSL2 开发机, :3000）
   │  Server Component 经 lib/api.ts（BFF，服务端环境变量 API_BASE_URL）
   ▼
Rust 后端单进程（WSL2 开发机, :8080）
   │  sqlx PgPool（内网直连，非经前端）
   ▼
外部 PostgreSQL 实例（公网云实例（连接参数见 .env），库名见 .env）
```

- 进程守护：**A 期不配置**（开发期前台运行，Ctrl+C 即停）；生产守护方式随部署上线期决定。
- 缓存中间件：已部署但 A 期不接入（ADR-A-004）。

**环境变量（`.env.example` 全部键名）**：

| 键 | 必填 | 说明 |
|----|------|------|
| `APP_ENV` | ✅ | `dev` / `prod`（基线 §12） |
| `APP_PORT` | ✅ | 后端监听端口（如 `8080`） |
| `DATABASE_URL_DEV` | `APP_ENV=dev` 时必填 | dev 库连接串 |
| `DATABASE_URL_PROD` | `APP_ENV=prod` 时必填 | 正式库连接串（A 期可不填，但键名保留） |
| `API_BASE_URL` | ✅ | 前端 BFF 指向后端的地址（仅服务端使用） |
| `NEXT_PUBLIC_APP_VERSION` | ❌ | 前端展示版本（缺省从 package.json 读） |
| `RUST_LOG` | ❌ | 默认 `info` |

**启动命令**：

```bash
# 开发（一条命令起前后端）
scripts/start.sh
# 内部步骤：工具链版本检查 → .env 存在性与必需键检查（缺失即 ERR-003 退出）
#          → 并行：后端 cargo run（先 cargo sqlx migrate run）/ 前端 npm run dev

# 类型生成（后端接口变更后）
scripts/gen-types.sh
# 内部步骤：起后端（或复用运行中实例）→ 抓 /api/v1/openapi.json（utoipaswagger）
#          → openapi-typescript 生成 web/src/lib/api-types.ts

# 质量门（提交前）
scripts/check.sh   # cargo clippy -D warnings && cargo test && biome check && tsc --noEmit
```

## 10. 测试计划（基线 §14.3 模板）

**业务规则测试**（service 层，`cargo test`）：

| 用例 | 对应 | 断言 |
|------|------|------|
| health_service：库可达 | RULE-001 / AC-8 | `db="ok"`，字段恰为 3 个 |
| health_service：库不可达 | RULE-001 / AC-8 | `db="error"`，HTTP 200，无内部信息泄露 |
| config：缺单键 / 缺多键 / dev-prod 同库 | RULE-002 / AC-2 | 启动失败，错误信息含全部缺失键名 |
| 信封序列化 | RULE-003 | 成功/失败信封结构与基线 §6.1 一致 |

**API 集成测试**（`sqlx::test` 或 mock pool）：`GET /api/v1/health` 正常 + 库故障两例；根路径 `/health` 同义。

**权限测试**：A 期无权限点（ADR-A-001），越权用例 B 期补。

**数据精度测试**：不适用（无金额计算）。

**前端手工验收清单**（对照 prd-a.md §8.3/§8.4 与 design-a.md）：

- [ ] P01 五态走查：正常 / 空态 / 加载骨架（节流 3G+300ms）/ 后端未启动 ERR-001 / 库停 ERR-002
- [ ] 暗色模式切换走查（AC-7）
- [ ] `prefers-reduced-motion` 下骨架静止（AC-9）
- [ ] 桌面 1280 / 1440 两档宽度无横向滚动
- [ ] `grep -rn "#[0-9a-fA-F]\{3,8\}\|px" web/src --include="*.tsx"` 无裸值（AC-6，globals.css 除外）
- [ ] codegen 链路：改后端字段 → 不重新生成 → `tsc --noEmit` 报错（AC-5）

**已知风险与规避**：RISK-001 ~ 005（prd-a.md §10.2）；执行顺序上**第一天先做 DEP-001 连通性验证**（telnet/psql 客户端探外部 PG 端口），不通则停下解决网络，不写代码。

---

## 11. A 期交付物清单与验收顺序

1. `server/`：编译零警告（clippy -D warnings）、`/api/v1/health` 与 `/health` 双挂、迁移链路通、`.sqlx/` 提交
2. `web/`：dev 可跑、P01 五态、`@theme` Token 全量、Biome/tsc 通过、`api-types.ts` 生成
3. `scripts/`：start.sh / gen-types.sh / check.sh 可用
4. `docs/`：本文件 + prd-a.md + design-a.md + test-report-a.md（开发完成后产出）
5. README.md：启动 / 类型生成 / 备份说明（备份指向 ADR-A-002）

验收顺序 = §11 顺序；每步通过再进入下一步（骨架期返工成本最低的做法是串行验收）。
