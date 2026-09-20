# 02 · T0 + 候选生成（3 direction 连发）

Status: needs-triage
Labels: ready-for-agent

> 前置：01 完成；open-design MCP 已重连（苑问跑 `/mcp`）。
> 授权：苑问已一次授权 3 连发（spec §2）；每次仍是 BYOK 真实调用，不批量重跑。

## T0 自查（AI，未达标不开跑）

- design system：`modern-minimal` / `tech-utility` / `human-approachable`（内置 direction）
- skillId：`redesign-existing-projects`；fidelity：`high-fidelity`
- customInstructions：`.scratch/od-redesign/custom-instructions.md`（已落盘 Git）✅

## 步骤

1. 3 个 direction 各建一个独立 OpenDesign project（评估性质，见 spec §6.7），对 P01 首页各生成一版小样
2. 产物一律视为草稿；生成即录草稿元数据（direction/skill/fidelity/日期/工具版本），暂记本 ticket Comments

## DoD

- 3 个候选 artifact 可预览，元数据齐全
