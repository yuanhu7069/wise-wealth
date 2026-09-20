# 04 · 固化 P01 胜者产物

Status: resolved
Labels: ready-for-agent

> 前置：03 完成（T1 通过）。基线 §4 P0：生成后立即固化，不攒批。

## 步骤

1. tokens 落 `web/src/app/globals.css`（沿用 Tailwind v4 `@theme` 结构与亮暗重组惯例；暗色 `--dark-*` 单点定义）
2. 产物副本 → `docs/design/`（HTML 快照）
3. 生成元数据 → `docs/design/生成记录.md`（direction/skill/fidelity/customInstructions 版本指针/生成日期/工具版本/迭代轮次）
4. 此后改动直接改代码，不回炉生成

## DoD

- `bash scripts/check.sh` 全绿；生成记录可回答「这套界面怎么来的」

## Comments

- 2026-09-20 · AI · **范围微调备案**:原步骤 1「tokens 落 globals.css」移入 ticket 05 落地——token 名是全局的,单独换值会让 P02–P04 的旧类名指向新值、P01 结构却还是旧布局,产生中间态不一致;随 ticket 05 一次性切换(值+组件同 PR)更干净。本 ticket 保留纯固化:快照 + 生成记录,已达成「产物进 Git」这一基线 P0。
- 2026-09-20 · AI · 固化完成:快照 `docs/design/od-redesign/p01-home-github.html`(637 行,sha256 前 16 位 31900488b5b77839,与 daemon 侧产物逐字节一致);生成记录 `docs/design/生成记录.md`(四件套/工具版本/run/候选过程/派生规则全录)。src/scripts 无改动,check.sh 维持 01e074a 的全绿状态(文档新增不参与任何检查项)。DoD 达成。
