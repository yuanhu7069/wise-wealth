# 04 · 固化 P01 胜者产物

Status: needs-triage
Labels: ready-for-agent

> 前置：03 完成（T1 通过）。基线 §4 P0：生成后立即固化，不攒批。

## 步骤

1. tokens 落 `web/src/app/globals.css`（沿用 Tailwind v4 `@theme` 结构与亮暗重组惯例；暗色 `--dark-*` 单点定义）
2. 产物副本 → `docs/design/`（HTML 快照）
3. 生成元数据 → `docs/design/生成记录.md`（direction/skill/fidelity/customInstructions 版本指针/生成日期/工具版本/迭代轮次）
4. 此后改动直接改代码，不回炉生成

## DoD

- `bash scripts/check.sh` 全绿；生成记录可回答「这套界面怎么来的」
