# 执行计划：智策理财 · A 期工程骨架（a-plan.md）

> 对应需求：REQ-20260907-01（prd-a.md）／架构：arch-a.md／设计：design-a.md
> 生成日期：2026-09-07　状态：**已完成，苑问验收通过（2026-09-08）**（T1–T6 全过，9/9 AC 真浏览器断言证据 + 苑问目检确认；遗留项 5 Token 样例区/免责声明区已按 design-a.md §1 补齐；测试报告见 docs/a/test-report-a.md）
> 执行者：AI（Claude Code）　验收人：苑问
> 执行原则：按任务串行推进，每个任务以「完成门」结束，门不过不进下一任务；对照基线 arch §16 / design §13 红线清单自查

---

## 0. 环境事实（2026-09-07 已验证）

| 项 | 状态 | 说明 |
|----|------|------|
| Rust | cargo/rustc 1.94.0 | 满足基线稳定版要求，rust-toolchain.toml 锁定此版本 |
| Node | v22.22.1 + npm 10.9.4 | 满足 Next 16 要求 |
| Git | 2.43.0 | 仓库尚未初始化（T1 执行） |
| PostgreSQL | 15.19（外部云实例（连接参数见 .env）） | psql 认证通过；已有 `wise_wealth_db`，A 期用作 dev 库 |
| TLS | ⚠️ **服务器拒绝 SSL**，当前明文传输 | 见 RISK-A-1，先记录不阻塞 |
| 已就位文件 | `.env`（真实凭证）/ `.env.example`（键名模板）/ `.gitignore` | RULE-002/007 的载体，T1 只需 `git init` 后确认不入库 |

**RISK-A-1（新增，记录于本计划）**：外部 PG 实例未启用 TLS，数据库口令明文过公网。缓解：① 不在本机 `.env` 之外留存凭证；② A 期库中不含任何业务数据；③ 建议苑问在服务器端启用 TLS 或改走 SSH 隧道（不阻塞 A 期，B 期引入真实数据前**必须**解决，届时回填 arch-a.md §6）。

## 1. 任务分解

### T1 · 仓库初始化
- `git init` + 初始提交（docs/ 三基线实例文档 + 产品 PRD + `.gitignore` + `.env.example`；**确认 `.env` 未被跟踪**）
- 建 `server/ web/ scripts/` 目录占位
- **完成门**：`git status` 不出现 `.env`；`git log` 有首个提交

### T2 · Rust 后端骨架（arch-a.md §3 结构）
1. `cargo init` + `rust-toolchain.toml`（锁 1.94.0）+ 依赖（仅基线 §1.1 清单内：axum/tokio/sqlx/serde/validator/thiserror/anyhow/tracing/tracing-subscriber/utoipa/dotenvy/uuid）
2. 骨架文件：`config.rs`（RULE-002 校验：缺键逐条列出后退出；dev/prod 同库检测）、`error.rs`（AppError + ErrorCode + 统一信封 IntoResponse，RULE-003）、`state.rs`（PgPoolOptions 显式最大连接数与超时）、`openapi.rs`（utoipa 装配）、`main.rs`（tracing 含 request_id、优雅退出）
3. `api/v1/health.rs` + `services/health_service.rs`（RULE-001：data 仅 status/db/version 三字段；`/api/v1/health` 与 `/health` 双挂，ADR-A-003 恒 200）
4. sqlx-cli 迁移链路：`sqlx migrate add 0001_init_baseline`（空基线迁移，UP/DOWN 可执行）→ `sqlx migrate run` → `cargo sqlx prepare`（`.sqlx/` 产物待提交）
5. **完成门**：`cargo clippy -D warnings` 零告警；`cargo run` 后 `curl /api/v1/health` 返回 RULE-001 信封；停库模拟 `db=error`；RULE-002 三条 config 单测绿（缺单键/缺多键/dev-prod 同库）

### T3 · Next.js 前端骨架（design-a.md 全量落地）
1. `create-next-app`（App Router + TS + Tailwind 4）；tsconfig 显式处理 TS7 硬变更（strict、types 显式列 `@types/node`、无 es5/baseUrl）；装 Biome（**不装 ESLint**，基线 §1.3）
2. `globals.css`：design 基线 §5.1.1/§5.1.2/§5.2/§5.3/§5.4/§5.5 逐 Token 抄入 `@theme inline`；暗色 media query 重定义；shadcn slate 色板全部映射苑问色板
3. shadcn 最小引入：button/card/skeleton/badge + lucide-react；主题覆盖后禁留默认色
4. `lib/api.ts`（BFF 唯一出口，服务端 `API_BASE_URL`）+ `lib/errors.ts`（errorCode→文案映射，RULE-008 三要素）
5. P01（`app/page.tsx` Server 取数 + 状态卡片 Client 子组件 + `loading.tsx` 骨架）：三区块（头部 / 状态卡 / Token 样例区 / 免责声明区=智策 PRD §13.1 原文）；五态按 design-a.md §2 三形态实现；文案唯一来源 prd-a.md §8.4/§7.1；图标按 design-a.md §5
6. **完成门**：`biome check` + `tsc --noEmit` 通过；Token 门禁 grep 零命中（裸色值/px 仅 globals.css）；DevTools 五态 + 暗色 + reduced-motion 走查（AC-3/6/7/9）

### T4 · 类型生成链路（AC-4/5）
1. `scripts/gen-types.sh`：抓后端 `openapi.json` → openapi-typescript → `web/src/lib/api-types.ts`（文件头生成物标注）
2. P01 类型改为从 `api-types.ts` 引用，删除手写重复类型
3. **完成门**：gen-types 一次成功；故意改后端字段不重新生成时 `tsc --noEmit` 报错（AC-5 现场验证后还原）

### T5 · 启动与质量脚本
- `scripts/start.sh`（版本检查 → .env 检查 → migrate run → 并行起前后端）、`scripts/check.sh`（clippy + cargo test + biome + tsc + Token 门禁）
- README.md：启动 / gen-types / check / 备份说明（备份指向 ADR-A-002）
- **完成门**：全新 shell 执行 start.sh ≤60s 双服务在线（AC-1）；check.sh 全绿

### T6 · 验收与收尾
1. 按 arch-a.md §10 测试计划跑完整门：RULE-001/002/003 单测、health 集成测试（正常+库故障）、五态/暗色/reduced-motion 手工走查
2. `cargo audit` + `npm audit` 结果记入 test-report
3. 产出 `docs/a/test-report-a.md`（基线 §16 清单逐项 ✅/❌/不适用+理由）
4. 变更记录：三份实例文档若无改动则保持；prd-a.md 状态改「已完成」前需苑问验收
5. **完成门**：DoD 全项满足，交苑问走查（含「停库→错误态→恢复」演练）

## 2. 执行顺序与依赖

```
T1 → T2 → T3 → T4 → T5 → T6（串行；T4 依赖 T2 的 openapi 与 T3 的工程）
```

预计 T2/T3 各占一半工作量；T6 的 test-report 产出后本计划才算闭环。

## 3. 交付物清单

- 代码：`server/`（含 `.sqlx/`、`migrations/`）、`web/`、`scripts/`（start.sh / gen-types.sh / check.sh）、README.md
- 文档：本文件状态更新 + `docs/a/test-report-a.md`
- Git：主干提交历史干净（每任务一提交，信息含任务号）

## 4. 验收标准回链（全部 AC 的覆盖方式）

| AC | 覆盖任务 | 验证方式 |
|----|---------|---------|
| AC-1 启动 | T5 | start.sh 计时 ×3 |
| AC-2 配置缺失 | T2 | config 单测 + 手动删键启动 |
| AC-3 后端不可达 | T3 | 停后端开 P01 |
| AC-4 类型生成 | T4 | gen-types.sh |
| AC-5 变更暴露 | T4 | 改字段→tsc 报错 |
| AC-6 Token 门禁 | T3/T5 | check.sh grep |
| AC-7 暗色 | T3 | DevTools 走查 |
| AC-8 健康信封 | T2/T6 | 单测 + 停库实测 |
| AC-9 动效降级 | T3 | reduced-motion 走查 |

## 5. 执行结果（2026-09-07 回填）

| 任务 | 状态 | 完成门证据 |
|------|------|-----------|
| T1 仓库初始化 | ✅ 完成 | 提交 1944d8a；`.env` 未跟踪（git check-ignore 验证） |
| T2 后端骨架 | ✅ 完成 | 提交 2cb4664；clippy 零告警、单测 10 passed、双挂 health 实测、停库 db=error 恒 200、RULE-002 逐条列键 exit 1 |
| T3 前端骨架 | ✅ 完成 | 提交 ba6592d；biome/tsc 通过、Token 门禁零命中、P01 五态代码落地 |
| T4 类型生成链路 | ✅ 完成 | 提交 fdf98d1；gen-types.sh 成功；AC-5 现场验证（改 db 字段→TS2367→还原）通过 |
| T5 启动与质量脚本 | ✅ 完成 | 提交 75fae99；AC-1 计时 ×3（2s/3s/0s）≤60s；check.sh 五段门全绿 |
| T6 验收与收尾 | ✅ 开发侧完成 | test-report-a.md 产出；cargo audit/npm audit 双 0 漏洞；§16.6 unwrap/expect 清理（提交见 git log）；停库→恢复演练通过；**浏览器侧走查清单交苑问** |

遗留事项（不阻塞验收，B 期处理）：RISK-A-1 无 TLS；TS7/openapi-typescript 并存工作态；触控热区 40px→44px 统一调整；shadcn CLI 网络受限改手写等价组件。详见 test-report-a.md §6。
