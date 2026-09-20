# od-redesign · OpenDesign 生成型界面重设计

Status: in-progress
基线: `docs/base_line/基线-design-OpenDesign-Web域.md`（Open Design MCP 生成型 v0.1）
关联: `CONTEXT.md`「设计治理（Open Design 生成型）」词汇 · stripe-restyle 期（旧视觉成果，本期退役其真源地位，视觉沉淀 git 历史）

## 1 背景与目标

设计基线由「参考设计型」切换为「Open Design MCP 生成型」（2026-09-20 生效）。本期基于**现有功能**用 OpenDesign 重新生成一版界面并落地，提升美观度。

**边界**：功能不增不减；接口、数据流、业务字段一律不动；只重设计视觉与布局（允许组件划分随产物调整）。

## 2 已定决策（grilling · 2026-09-20，苑问逐项拍板）

| 决策点 | 结论 |
|---|---|
| design system | 内置 direction 对比法：`modern-minimal` / `tech-utility` / `human-approachable`（brutalist 排除，editorial-monocle 备选） |
| skillId | `redesign-existing-projects`；备选 `frontend-design`，切换作为参数变更记入生成记录 |
| fidelity | `high-fidelity`（源码确认枚举仅 wireframe / high-fidelity） |
| customInstructions | 本目录 `custom-instructions.md`，已定稿 |
| 试点策略 | P01 首页先行；P04 → P03 → P02 铺开，loading/404 顺带 |
| 落地深度 | 允许布局结构与组件调整，跟随产物 |
| 旧真源处置 | DESIGN.md 搬 `docs/design-ref/`；check.sh 摘除 `designmd validate`；`design-lock.json` 删除；`check-tokens.sh` 保留（代码卫生门禁，与真源无关） |
| 品牌色 | 不预锁；对比后若需对齐品牌绿，进调整清单 |
| 期管理 | `.scratch/od-redesign/` + 期文档 `docs/d/`；生成记录与快照进 `docs/design/` |
| 授权节奏 | 候选 3 连发一次授权（已获）；铺开阶段每页正式生成前打招呼 |

## 3 流程（三道闸门，每页一轮）

```
T0 参数闸门（AI 自查） → 生成（BYOK 真实调用） → T1 草稿验收 → 固化进 Git → 落地代码 → T2 落地验收
```

- **T0**：design system / skillId / fidelity / customInstructions 四件套已固定，customInstructions 已落盘本目录。未达标不得开生成。
- **T1 · A 一致性**：产物 token 与其声明的 direction 逐项比对，可自动的自动跑。
- **T1 · B 完整性**（六项，苑问按表勾）：五态齐全 / 极端数据不破版 / 中文渲染 / 自包含 / 无障碍 / 数值展示。**未过 T1 不得落地。**
- **固化**：tokens 进 `globals.css`（`@theme`）、产物副本进 `docs/design/`、元数据进 `docs/design/生成记录.md`。**生成后立即固化，不攒批。**
- **T2**：实现与固化产物的偏离必须能在 §5 调整清单找到，找不到即违规。CI 只跑 T2，不放生成。

## 4 调整清单（落地时逐项填，格式：调整项 | 生成原值 | 本项目值 | 理由）

> 起草：AI（草稿过闸后）；拍板：苑问。清单外一律照产物执行。

### 4.1 胜者产物（GitHub 风 · P01）调整清单初稿

| # | 调整项 | 生成原值 | 本项目值 | 理由 |
|---|--------|---------|---------|------|
| 1 | 正文字重 | 400（GitHub 二值制 400/600） | 500 | 中文细体在 Windows 偏糊，加重一档（customInstructions） |
| 2 | 正文行高 | 1.5 | 1.75 | 中文正文行高放宽（customInstructions） |
| 3 | 标题字距 | Display −0.01em | 0 | 中文标题不用负字距（customInstructions） |
| 4 | 字体栈 | 纯 system-ui | 追加 PingFang SC / Hiragino Sans GB / Microsoft YaHei | 中文回退（customInstructions） |
| 5 | 按钮字距 | normal | 0.02em | 产物为 14px 小字按钮加的微字距；倾向保留，苑问可收回 |
| 6 | 卡片内距 | 20px（产物 tokens 合同 space-5） | 16px（p-base-lg） | 沿用现有 8px 基刻度近似产物节奏；改动全局刻度会波及未落地页 |
| 7 | 错误卡图标 | 产物手绘 svg 三角 | lucide `AlertTriangle` | 全站图标统一 lucide 库，形态一致 |
| 8 | 加载骨架 | 产物仅摘要卡骨架 | 保留 hero 骨架 + 摘要卡骨架 | 真实页面有 hero，路由级 loading 需要占位（结构增强，非视觉偏离） |

### 4.2 派生规则记录（DS 未覆盖区，非偏离，固化时随生成记录存档）

- 暗色 accent/success/warn/danger：DS §2 Dark Theme 只文档化 4 值（画布/表面/边框/前景），产物以 `color-mix` 从文档化亮色值机械派生
- 暗色主按钮 `#238636` / hover `#2ea043`、danger hover `#da3633`：GitHub 实际产品暗色值，DS 文档未载，产物直接采用

### 4.3 产物静态值 → 落地时接动态（落地说明，非视觉偏离）

- 页脚版本号 `v1.2.0` → 接真实构建版本（P01 已接：`v{version}` 来自 `/api/v1/health`）
- 页脚「服务正常」健康点 → 接 `/api/v1/health` 动态探测（沿用 SiteFooterShell 降级黄点机制）
- 首页 hero 的「P01 · 产品首页」eyebrow 为产物自注标注，非产品文案，未落地
- 字体：产物 system 栈（GitHub「System fonts always」），原 Inter webfont 已随落地移除（产物忠实，非偏离）

### 4.4 全局刻度说明（ticket 05 落地决策）

- **token 名全数保留、值切 GitHub 契约**：P02–P04 即刻整体换肤（颜色/字阶/圆角/阴影随类名生效），各页结构适配在其 ticket（06–08）内完成
- 间距刻度沿用 8px 基八档不变（理由同 #6）；圆角 token 不变，组件类名改用 `rounded-sm`（6px）
- 图表六色：DS 未定义图表序列，按语义色族机械映射（蓝/绿/紫/黄/灰/红），见 §4.2 同款规则

## 5 人工环节分布

| 环节 | 执行者 |
|---|---|
| direction 对比拍板 | 苑问（并排对比 3 候选，选"别扭最少"） |
| T1·B 完整性六项勾选 | 苑问（自动项 AI 先跑：自包含、对比度脚本） |
| 调整清单拍板 | 苑问 |
| 每页正式生成前授权 | 苑问（铺开阶段） |

## 6 AI 硬约束

1. 参数未给全不开生成；2. 服务不可用（含 MCP 断连）停下来报告，不降级不换来源；3. 迭代对话式修改，不整项目重生成；4. 每次生成后立即固化并补生成记录；5. 调整清单外不按自己审美改动；6. 草稿未过 T1 不落地，不把 happy path 当完成品；7. 候选阶段 3 个独立 project（评估性质），选定后胜者 project 为唯一迭代载体，不再新建；8. 不批量重跑生成，每次都是真实调用。
