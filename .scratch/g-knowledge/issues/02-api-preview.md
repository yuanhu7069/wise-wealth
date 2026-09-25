# 02 · API 层：knowledge ×2 端点 + plans/preview + PageId 扩展

Status: open
Labels: ready-for-agent

> g-plan T2。前置：01 完成。规格：arch-g.md §0 ADR-G-002/004、§3、§4、§5。

## 步骤

1. `api/v1/knowledge.rs`：GET `/knowledge`（元数据列表，**不含 sections**，kind 分组序）+ GET `/knowledge/:id`（全文；未知 id → 404 信封）；随建随挂 ApiDoc
2. `POST /plans/preview`（ADR-G-002）：请求 `{mode_ids: [1..=3]}`；档案不完整 → 422（与生成同口径）；逐 id 调 `engine::solve`——**零写库调用（评审清单项）**；响应逐模式 `{mode_id, ok, meta{name,credibility,source,tagline,fit_for,buckets[{name,share_desc}]}, solution{buckets[],l2_name,emergency,notices}|null, reason?}`；非法 id/超量 → 422
3. **对账单测**：同档案同模式下 preview.solution 与生成路径 solve 结果逐字段一致（AC-6 的自动化底座）；不可行金例（收入 800/固定 700）→ ok=false + reason
4. `PageId` 枚举 +P07/P08；客户端白名单同步
5. `api-types.ts` 手工移植（ADR-G-004：以 `curl /api/v1/openapi.json` 实际输出为准，逐字段对照，工单 Comments 记录移植清单）；tsc 校验
6. 集成测试：3 新端点无 Cookie → 401；超量/未知 id → 422/404

## DoD

- `cargo test` 全绿；clippy 零警告；check.sh 全绿
- 对账单测绿（preview==solve）；curl 实测三端点响应符合 arch-g §4
- 移植清单记录在案；确认零迁移、零 crate、`.env` 零新增
