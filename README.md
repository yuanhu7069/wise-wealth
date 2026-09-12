# 智策理财(wise-wealth)

双层理财决策工具:**免费 Guide 层告诉你「怎么做」,Plus 层告诉你「为什么是这个数」以及「换个条件会变成什么」**。

只输出建议与模拟,**不触碰资金** —— 不托管、不代客操作、不接券商或基金账户,也不出现具体金融产品名称或购买链接。

## 产品是什么

- **Guide(免费 · 默认入口)**:从模式库挑一套有出处的资金安排方法,答一份引导问卷,拿到一张可照做的方案表;方案可重新生成,历史版本保留。
- **Plus(进阶 · 未实现)**:展开「这个数字怎么来的」的推理链,假设参数可见、可调、可存多套。

模式分三层:L1 回答「钱进来先切成几堆」,L2 回答「投资的钱装什么大类、各占多少」,L3 回答「怎么买、什么时候调」;
每个模式的出处都带三级可信度评级。领域词汇以 [CONTEXT.md](./CONTEXT.md) 为准。

## 已实现的流程

登录 → 引导问卷(6 步一屏一题) → 方案(五段静态渲染),外加首页的空态 / 摘要双形态。

| 位置 | 内容 |
|------|------|
| 页面 | `/` 首页 · `/login` 登录 · `/questionnaire` 问卷 · `/plan` 方案 |
| 后端 | 健康检查 · 会话鉴权 · 模式查询 · 方案生成 |
| 数据 | PostgreSQL(迁移在 `server/migrations/`) |

账号是单账号、无注册页:登录凭证由本地 `.env` 的种子账号决定,怎么设见 [DEPLOY.md](./DEPLOY.md)。

## 快速开始

```bash
cp .env.example .env    # 填 DATABASE_URL_DEV / DATABASE_URL_PROD、JWT_SECRET、SEED_USERNAME / SEED_PASSWORD
scripts/seed-user.sh    # 首次登录前跑一次:把 .env 里那对账号写进数据库
scripts/start.sh        # 后端 :8080 + 前端 :3000
```

然后打开 http://localhost:3000,用 `.env` 里那对账号登录。

环境要求、质量门、类型生成、构建、备份与恢复演练、升级步骤都见 **[DEPLOY.md](./DEPLOY.md)**。

## 技术栈

| 层 | 技术 |
|----|------|
| 后端 | Rust 1.94(axum 0.8 + sqlx 0.9),监听 :8080 |
| 前端 | Next.js 16 App Router + Tailwind 4,监听 :3000 |
| 数据库 | PostgreSQL 15 |
| 契约 | 后端 OpenAPI → `openapi-typescript` 生成 `web/src/lib/api-types.ts`(**生成物,禁止手改**) |
| 质量 | clippy -D warnings · cargo test · biome · tsc --noEmit · Token 门禁,由 `scripts/check.sh` 一次跑全 |

## 目录结构

```
server/    Rust 后端(迁移在 server/migrations/)
web/       Next.js 前端
scripts/   启动 / 质量门 / 备份 / 恢复演练 / 种子账号
docs/      产品 PRD、各期实例文档、工程基线
DEPLOY.md  部署与运维
CONTEXT.md 领域词汇表
```

## 文档在哪

| 想知道什么 | 看哪 |
|------------|------|
| 怎么把它跑起来、怎么备份与恢复、怎么升级 | [DEPLOY.md](./DEPLOY.md) |
| 产品要做什么 | `docs/智策理财_PRD_V1.1.md` |
| 术语的准确定义 | [CONTEXT.md](./CONTEXT.md) |
| 各期的需求 / 设计 / 架构 / 验收记录 | `docs/a/`(工程骨架)、`docs/b/`(Guide 首流程) |
| 工程基线:质量门、安全与合规 | `docs/base_line/` |
| Agent 协作约定 | `AGENTS.md`、`docs/agents/` |
