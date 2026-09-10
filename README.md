# 智策理财(wise-wealth)· A 期工程骨架

不碰资金的理财决策工具。A 期只交付工程骨架:Rust 后端 + Next.js 前端 + 类型生成链路 + 质量门脚本。
计划与验收见 `docs/a/a-plan.md`,架构/设计/需求实例见 `docs/a/`。

## 目录结构

```
server/   Rust 后端(axum 0.8 + sqlx 0.9 + PostgreSQL,监听 :8080)
web/      Next.js 16 前端(App Router + Tailwind 4 + shadcn 结构组件,监听 :3000)
scripts/  start.sh / check.sh / backup.sh / restore-drill.sh / seed-user.sh / web 内 gen-types.sh、check-tokens.sh
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
- 浏览器访问:优先用 `http://localhost:3000`;若必须用 `127.0.0.1`,把 `127.0.0.1`/`localhost`
  加入系统代理(Clash 等)的绕行名单,否则页面会打不开(代理返回 502)
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

## 构建(RULE-004 交付形态)

```bash
cd web && npm run build      # next build(Turbopack),产物 .next/
cd web && npm run start      # 生产模式启动(需先 build)
cd server && SQLX_OFFLINE=true cargo build --release   # 后端 release 二进制
```

A 期为工程骨架期,构建仅验证可通过、无运行部署目标;B 期起按 PRD 交付形态补部署说明。

## 数据库备份与恢复(ADR-A-002 / arch-v2 §7)

> ⚠️ **恢复演练未通过前,不录入不可重建的真实数据**(prd-v1 红线 4)。
> 本次已于 **2026-09-11 通过**(AC-15,兑现 ADR-A-002):10 项抽验一致、副本库上服务 `db=ok`、
> 种子账号可登录并读出方案,记录见 `docs/b/test-report-b.md` §1.2。
> **换库、改迁移脚本或改恢复链路后,红线重新生效** —— 先重跑 `scripts/restore-drill.sh` 再录真实数据。

### 备份

```bash
scripts/backup.sh              # pg_dump 自定义格式 → backups/ + .sha256 校验文件
scripts/backup.sh --keep-all   # 只备份不清理(归档留档时用)
```

- 产出:`backups/wise_wealth_YYYYMMDD_HHMM.dump`(自定义格式,恢复时可选择/校验)+ 同名 `.sha256`
- 保留策略:**最近 7 个日历日各留最新一份 + 最近 5 个 ISO 周各留最新一份**,其余自动清理。
  回退窗口 ≥ 4 周(实测最老一份 ≥29 天),满足基线「至少能回退到 7 天前与 1 个月前」。
  判据取**文件名里的时间戳**,不取 mtime —— 拷贝与同步会改 mtime,文件名才是备份的身份;
  一天跑多次也只留当天最新那份(原先「周备 4 份」只能保证 21 天,故加一周)
- `backups/` 已 gitignore:**备份文件不进版本库**
- 备份只在本机磁盘,挡不住本机磁盘故障:**请手工拷一份到云盘或另一台机器**(异地副本)
- 目标库按 `.env` 的 `APP_ENV` 选:dev → `DATABASE_URL_DEV`,prod → `DATABASE_URL_PROD`
- 每份备份另附同名 `.meta`:时间、库名、git 版本(含是否有未提交改动)、已应用迁移数,
  以及**脱敏配置快照**(APP_ENV / APP_PORT / API_BASE_URL 等非密钥项照记,密钥只记「已配置」)
  —— 基线 §7.3 的备份内容 = 数据库 + 配置(不含密钥)+ 代码版本
- 两个可选开关:`BACKUP_DIR`(备份目录,缺省 `backups/`)、`WW_DRILL_DB`(副本库名,缺省 `wise_wealth_db_test`)

### 恢复与演练(同一条路径)

```bash
scripts/restore-drill.sh                    # 先备份,再拿这份备份演练
scripts/restore-drill.sh backups/xxx.dump   # 演练指定备份(「从旧备份恢复」场景)
```

演练脚本做四件事:①校验 SHA256(没有校验值的备份不算备份)②恢复到副本库
(`WW_DRILL_DB`,缺省 `wise_wealth_backup_test`)③抽验行数、金额合计与内容指纹与源库**逐项一致**
④用副本库起一次服务,`/health` 返回 `db=ok` 且能读出方案数据。全程不写源库。

真的需要恢复时,手工命令与脚本同款:

```bash
pg_restore --clean --if-exists --no-owner --no-privileges --dbname="$DATABASE_URL_DEV" backups/xxx.dump
```

副本库需要 CREATEDB 权限;当前角色没有该权限时,脚本会打印需要管理员执行的 SQL 并以退出码 3
结束 —— 它不会降级成「恢复进源库」,那等于用恢复演练制造一次事故。

## 升级

```bash
scripts/backup.sh && git pull && scripts/start.sh
#  ① 先备份:迁移会改结构,回退靠的就是这一份(基线 §7.2「执行前先备份」)
#  ② start.sh 会先跑 sqlx migrate run 再起服务
```

迁移是向前兼容的增量(`server/migrations/`,每份都带 `.down.sql`);**改过迁移或换过库之后,
先跑一次 `scripts/restore-drill.sh` 再录真实数据** —— 恢复链路变了而没复验,等于没有备份。

## 常用命令

```bash
cd server && SQLX_OFFLINE=true cargo test    # 后端单测(离线)
cd server && cargo sqlx prepare              # 更新 .sqlx 查询缓存
cd web && npm run lint                       # biome
cd web && npm run typecheck                  # tsc --noEmit
scripts/backup.sh                            # 备份数据库(自定义格式 + SHA256 + 保留策略)
scripts/restore-drill.sh                     # 恢复演练(副本库抽验 + 副本库起服务验证)
```
