# 05 · 验收收尾：AC 全量 + 闸门断言 + 埋点 SQL + restore-drill + test-report-h + 期收口

Status: ready-for-agent
Labels: ready-for-agent

> h-plan T5。规格：prd-h DoD、§9.5；arch-h §10/§11；h-plan §2 验收顺序、§3 收口清单。

## 步骤

1. AC-1～14 全量走查（free/plus 双态 × 375px/桌面 × 亮/暗），逐条记录结果
2. **闸门双向断言复验**：free 会话 curl 方案接口 → 响应体无 `traces` 键（仅 `trace_count`）；`set-tier.sh` 置 plus → 刷新 P04 即见全链（无重新生成）；回置 free → 复验占位恢复（AC-4 全链路）
3. 引擎改动审计：`git diff` engine.rs 仅让位 trace 追加；既有金例 + preview 对账回归记录在案
4. 埋点 SQL 抽验：`page_view`（page_id=p09）入库可查，列出事件名；金额/链条内容零命中埋点与日志（RULE-052）
5. restore-drill 复跑（无增项）全绿；可一键启动
6. 产出 `docs/h/test-report-h.md`：红线逐项 ✅ + AC 回链 + P09 闸门记录（T0/T1/T2）+ 引擎审计 + 双态走查记录
7. 期收口（h-plan §3 清单逐项勾）：prd-h 状态回填、生成记录齐备、零迁移/零 `.env` 新增核对、挂账核对（金字塔 / gen-types.sh TS7 / TLS / 其余延续项）、**苑问 DoD 验收动作**（真实档案展开推理链 + 录 10 月快照看 Q4 复盘）记录 → 收口提交

## DoD

- prd-h DoD 全项 ✅（功能/体验/质量/设计/数据/运维/验收动作）
- test-report-h 落盘；期收口提交完成

## Comments

-
