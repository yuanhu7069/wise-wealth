# 智策理财(wise-wealth)· 部署与运维

> 本文是**运行手册**:环境准备 → 首次配置 → 启动 → 质量门 → 类型生成 → 构建 → 备份与恢复 → 升级 → 常用命令。
> 产品与项目简介见 [README.md](./README.md),领域词汇见 [CONTEXT.md](./CONTEXT.md)。
> 本文自 `README.md` 拆出(2026-09-11):README 只留产品简介与最小快速开始,运行细节集中在这里。
> 文中的 `AC-*` / `ADR-*` / `RULE-*` / 基线 § 编号指向 `docs/` 下的需求、架构与验收记录,保留以便回溯。

## 目录结构

```
server/    Rust 后端(axum 0.8 + sqlx 0.9 + PostgreSQL,监听 :8080;迁移在 server/migrations/)
web/       Next.js 16 前端(App Router + Tailwind 4 + shadcn 结构组件,监听 :3000)
scripts/   start.sh / check.sh / backup.sh / restore-drill.sh / seed-user.sh,以及 web 内 gen-types.sh、check-tokens.sh
docs/      产品 PRD、各期实例文档(docs/a、docs/b)、工程基线(docs/base_line)、Agent 约定(docs/agents)
DEPLOY.md  本文:部署与运维
CONTEXT.md 领域词汇表
```

## 环境要求

- Rust 1.94.0(`server/rust-toolchain.toml` 锁定)、`sqlx-cli`
- Node ≥22、npm ≥10
- PostgreSQL 15(连接信息在 `.env`,**禁止提交入库**)

## 首次配置

```bash
cp .env.example .env
# 编辑 .env:
#   DATABASE_URL_DEV / DATABASE_URL_PROD(APP_ENV=dev 时要求 DEV 库与 PROD 库不同)
#   JWT_SECRET(会话签名密钥,≥32 字符,如 openssl rand -base64 32)
#   SEED_USERNAME / SEED_PASSWORD —— 登录账号,见下节
scripts/seed-user.sh   # 把上面那对账号写进数据库(必须在首次登录前跑一次)
```

## 登录与账号(AC-1 / ADR-B-002)

**没有注册页,也没有内置默认密码**;登录账号由 `.env` 的 `SEED_USERNAME` / `SEED_PASSWORD`
经种子脚本写入数据库,再在登录页 `/login` 用这一对值登录:

```bash
scripts/seed-user.sh            # 幂等:账号不存在则新建,存在则只重置口令
```

- 用户名与口令**就是 `.env` 里那两个值**:它们只存在于本地 `.env`(已 gitignore),
  不进版本库、也不写进本文档 —— 想知道是什么就打开 `.env` 看,或重新设一对再跑脚本
- 忘记口令:改 `.env` 的 `SEED_PASSWORD` 后重跑 `scripts/seed-user.sh`(口令只进 argon2 哈希,
  含启动日志在内的任何路径都不打印原文)
- **没跑种子脚本 → 页面能开但登不进去**,因为库里根本没有这个账号(报错是「用户名或密码不正确」)
- 未登录访问任何业务页面 → 跳 `/login?from=<原路径>`,登录后回到原处;是会话过期(带着旧 Cookie
  被拒)则额外带 `&expired=1`,登录页提示「登录已过期」
- 连续输错 5 次后,第 6 次起 1 分钟内返回 429(页面提示「操作太频繁,稍后再试」)
- 登录后的顶部栏(品牌名回首页、主题三态切换、登出)属交互设计,见 `docs/b/design-v2.md`
  §1.3、§3.3;本文不重复。排查用的实现锚点:主题偏好存 `localStorage['ww-theme']`
  (`web/src/app/layout.tsx`、`web/src/components/theme-toggle.tsx`),首帧前由内联脚本应用

## 启动(AC-1:≤60s 双服务在线)

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
- 种子账号不在 start.sh 里:首次使用仍须先跑一次 `scripts/seed-user.sh`(见上节)

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

## 构建与部署形态(RULE-004)

```bash
cd web && npm run build      # next build(Turbopack),产物 .next/
cd web && npm run start      # 生产模式启动(需先 build)
cd server && SQLX_OFFLINE=true cargo build --release   # 后端 release 二进制
```

当前部署形态:WSL2 上跑双服务(后端 :8080 + 前端 :3000)连外部 PostgreSQL;
**dev 库与 prod 库分离仍延后,当前 dev 库即业务库**(RISK-B-1);尚无对外部署目标,
对外前必须解决 TLS 与独立正式库(见 `docs/b/prd-v1.md` RISK-B-1)。

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
(`WW_DRILL_DB`,缺省 `wise_wealth_db_test`)③抽验行数、金额合计与内容指纹与源库**逐项一致**
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
