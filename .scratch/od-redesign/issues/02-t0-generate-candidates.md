# 02 · T0 + 候选生成（3 direction 连发）

Status: resolved
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

## Comments

- 2026-09-20 · AI · T0 通过:四件套固定 —— design system: `linear-app` / `github` / `duolingo`(原定 direction 三角映射到品牌库正主:modern-minimal→Linear、tech-utility→GitHub、human-approachable→Duolingo;start_run 可挂载的是品牌 DESIGN.md 库,direction 是绑 :root 的姿态规格,故取映射。审美三角不变,T1·A 逐值比对有了具象基准);skillId `redesign-existing-projects`;fidelity high-fidelity(写入提示词);customInstructions 内容全文内联进生成提示词,源文件 .scratch/od-redesign/custom-instructions.md @ 01e074a。
- 2026-09-20 · AI · 3 run 已连发(一次授权):项目 ww-redesign-linear / run f660e590-f5c4-4b07-8737-54ab3041a45d;ww-redesign-github / f1c337c3-e8b2-4fd7-a1e9-c0ba95267d13;ww-redesign-duolingo / 159443ba-d5d5-4114-b62c-7eb73cdaab02。工具版本 0.22.1(daemon 7456)。提示词含 P01 真实文案三形态+加载骨架+亮暗双主题+自包含单文件 HTML 交付要求。
- 2026-09-20 · AI · 三 run 全部 succeeded(Linear 26.0min / GitHub 21.9min / Duolingo 31.5min),产物均单文件自包含 HTML、静态校验 pass:ww-redesign-linear/wise-wealth-p01-home.html(689 行)、ww-redesign-github/p01-home.html(637 行)、ww-redesign-duolingo/p01-home.html(531 行)。并排对比页 .scratch/od-redesign/shots/compare.html。DoD 达成,转 ticket 03(苑问拍板 + T1)。
