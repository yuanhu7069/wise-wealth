# 02 · API 层：snapshots ×5 端点 + DTO 校验 + 埋点扩展

Status: resolved
Labels: ready-for-agent

> e-plan T2。规格：arch-e.md §4/§5/§8；ADR-E-004（CSV 直出 text/csv，信封显式例外）。

## 步骤

1. `api/v1/snapshots.rs`：GET `/snapshots`（limit/offset，默认近 24，红线 11）、PUT `/snapshots/:month`、DELETE `/snapshots/:month`（仅最新月）、GET `/snapshots/export`、GET `/plans/active/export`
2. `dto/snapshot_dto.rs` 校验：month 格式 YYYY-MM 且 ≤ 当前月；金额两位小数与 |值| < ¥100 亿；balances 键集 == active 方案桶集（422）
3. GET `/snapshots` 响应带 `summary`（调 tracking 纯函数：persisted_months / emergency / latest.deviations）
4. CSV 导出：UTF-8 BOM + `Content-Disposition: attachment` + 文件名含日期；`csv_escape()` 接线（票 01 已落函数与单测）
5. 埋点：服务端 `snapshot_submit`（payload is_overwrite/is_special）/ `snapshot_delete` / `export_csv`（export_type）；客户端白名单扩 `snapshot_skip` + `PageId::P05`（p05）；金额零入 payload
6. 重跑 `scripts/gen-types.sh`；确认产物含存量六端点 + 新端点类型
7. 集成测试：新端点无 Cookie ×5 → 401；`month=2099-01` → 422；删除非最新月 → 422；埋点失败不阻断

## DoD

- `bash scripts/check.sh` 全绿
- curl 无 Cookie 逐端点 401；有 Cookie 导出响应头/正文前 3 字节 BOM 验证
- `web/src/lib/api-types.ts` 重生成后 tsc 无错

## Comments

- 2026-09-22 · AI · 完成。×5 端点 + 路由 + openapi 注册(全文档 12 路径);`parse_yuan_to_cents` 手写十进制(0.29 金例防 float 路线),`render_csv` BOM+CRLF+公式前缀拦截且**纯数字单元格绕过转义**(保住 Excel 数值类型);埋点 +4 事件(snapshot_submit/delete/skip/export_csv)+ PageId::P05;新查询 10 条 `cargo sqlx prepare` 落离线缓存。无 Cookie 401 ×5、未来月 422(不触库)、非法月份 422 均有路由级测试;check.sh 全绿(134 测试 + clippy -D warnings)。删除非最新月的 422 依赖真库,留待票 06 端到端(C 核 AC-10)。
- 2026-09-22 · AI · **工具链坑(备案)**:web/node_modules 的 typescript 已被升到 7.0.2(TS7 Go 版无 `ts.factory` JS API),`gen-types.sh` 里的 `npx openapi-typescript` 直接崩(TypeError: createKeywordTypeNode)——上次成功生成是 B 期(ts5 时代)。解法:用隔离环境 + A 期 vendor 的 ts5 包(`web/vendor/ts5/typescript-5.9.3.tgz`)临时 pnpm add openapi-typescript + ts5 → 用其 bin 生成。本次产物已入 Git;**gen-types.sh 的长期修法待苑问拍板**(候选:vendor ts5 常驻隔离脚本 / openapi-typescript 降级 / 换生成器),本期不改脚本。
