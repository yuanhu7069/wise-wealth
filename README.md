# 智策理财(wise-wealth)· A 期工程骨架

不碰资金的理财决策工具。A 期只交付工程骨架:Rust 后端 + Next.js 前端 + 类型生成链路 + 质量门脚本。
计划与验收见 `docs/a/a-plan.md`,架构/设计/需求实例见 `docs/a/`。

## 目录结构

```
server/   Rust 后端(axum 0.8 + sqlx 0.9 + PostgreSQL,监听 :8080)
web/      Next.js 16 前端(App Router + Tailwind 4 + shadcn 结构组件,监听 :3000)
scripts/  start.sh / check.sh / web 内 gen-types.sh、check-tokens.sh
docs/     基线实例文档与 A 期计划
```

## 环境要求

- Rust 1.94.0(`server/rust-toolchain.toml` 锁定)、`sqlx-cli`
- Node ≥22、npm ≥10
- PostgreSQL 15(连接信息在 `.env`,**禁止提交入库**)

## 首次配置

```bash
cp .env.example .env
# 编辑 .env:DATABASE_URL_DEV / DATABASE_URL_PROD(APP_ENV=dev 时要求 DEV 库与 PROD 库不同)
```

## 一键启动(AC-1:≤60s 双服务在线)

```bash
scripts/start.sh              # 后端 :8080 + 前端 :3000
scripts/start.sh --no-frontend
```

start.sh 流程:工具版本检查 → `.env` 存在检查(RULE-002:配置缺失由后端逐条列出键名后退出)→ `sqlx migrate run` → 并行起前后端 → 就绪探测。

- 健康检查:`curl --noproxy '*' http://127.0.0.1:8080/api/v1/health`
  (开发机若配了 `http_proxy`,必须加 `--noproxy '*'`,否则代理会劫持 127.0.0.1 返回 502)
- ADR-A-003:进程存活恒 HTTP 200,数据库状态在 `data.db`(`"ok"|"error"`),**不会**因库故障退出服务进程。

## 质量门

```bash
scripts/check.sh   # clippy -D warnings + cargo test + biome + tsc --noEmit + Token 门禁
```

Token 门禁(RULE-006):`web/src` 下禁止裸色值与 px 字面量,一律走 `globals.css` 的 Token;`web/scripts/check-tokens.sh` 可单独执行。

## 类型生成(RULE-005)

```bash
cd server && cargo run &        # 需要后端在线
cd web && ./scripts/gen-types.sh   # openapi.json → src/lib/api-types.ts
```

`src/lib/api-types.ts` 为生成物,**禁止手改**;后端字段变更后重新生成,前端引用处会在 `tsc --noEmit` 编译期暴露。

## 数据库备份(ADR-A-002)

A 期迁移链仅 `0001_init_baseline`(占位)。真实备份策略(每日全量 + WAL)随 B 期引入;
当前手工备份命令:`pg_dump "$DATABASE_URL_DEV" > backup.sql`(从 `.env` 取连接串,勿写死)。

## 常用命令

```bash
cd server && SQLX_OFFLINE=true cargo test    # 后端单测(离线)
cd server && cargo sqlx prepare              # 更新 .sqlx 查询缓存
cd web && npm run lint                       # biome
cd web && npm run typecheck                  # tsc --noEmit
```
