# 规格：Stripe 参考设计风格切换（Web 域 design 基线换代）

Status: ready-for-agent
Labels: ready-for-agent
Feature: stripe-restyle
治理基线：`docs/base_line/基线-design-参考设计-Web域.md`（v0.1，本项目自 2026-09-17 改版起生效）
视觉真源：根目录 `DESIGN.md`（改造件）；参考件 `docs/design-ref/DESIGN.stripe.md`（**非真源，只读**）
退役基线：`docs/base_line/基线-design-Web域.md`（自有风格，本项目不再遵循；文件保留——外置主档镜像 + docs/a、docs/b 历史期文档仍引用它）
领域词汇：根 `CONTEXT.md`（本次纯表现层改造，无领域变更、无 UI 词汇入表）

---

## Problem Statement

智策理财当前视觉是自有风格（松墨绿 `#4A6B57` + 暖纸底），功能已可用但观感不够专业。目标是以 Stripe 为参考做一轮风格切换：token 全量换代 + 组件形态重排，让界面获得金融基建产品的可信感。

这是 `基线-design-参考设计-Web域.md`（参考设计引入型治理基线）在本项目的首次落地：参考件提供全部视觉值，本 spec 只做三件事——来源可追溯（§2.1）、偏离可解释（§2.2）、改造件不被上游覆盖（§2.3）。

## Solution

按治理基线 §3 的工具链落地：

1. 参考件已拉取：`npx designmd.sh add voltagent/awesome-design-md/design-md/stripe --output docs/design-ref/DESIGN.stripe.md`，锁源 `design-lock.json`（仓库根，入库）；
2. 改造件 `./DESIGN.md`（项目根）：以参考件为底做 §2.2 本地化调整，文末附精简版调整清单（AI 可见性要求）；`designmd.sh validate` 挂入 `scripts/check.sh` 门禁；
3. `web/src/app/globals.css` 逐 Token 抄入改造件（亮/暗双套），token 命名采用参考件命名；
4. 组件层与四页重排：pill 按钮、hairline 卡片 + 蓝调阴影、Inter 字体、首页渐变 mesh hero；
5. 验收：check.sh 全绿 + 亮/暗两态人工走查。

## 决策记录（2026-09-17 grilling 共识，苑问确认）

| 决策点 | 结论 |
|---|---|
| 改造深度 | 标准重排（token + 组件形态），不做营销级版式重构 |
| 主色 | 全换 Stripe 靛蓝 `#533AFD`，松墨绿退役 |
| 涨跌色 | 红涨绿跌保留（域约定，非装饰） |
| 暗色模式 | 保留三态机制，暗色值本地派生（参考件无暗色） |
| 字体 | Inter via next/font（自托管）；中文字重加档 |
| mesh | 仅首页 hero（inline SVG）；P03/P04 产品页不加 |
| token 命名 | 采用参考件命名；仓库特有语义 token 保留原名 |
| 流程 | 本 spec + issues 走 `.scratch/` 流程；安全网 = 既有门禁 + 人工走查 |

---

## §2.1 来源追溯（治理基线模板）

| 项 | 填写 |
|----|------|
| 主参考来源 | `voltagent/awesome-design-md/design-md/stripe`（designmd.sh registry） |
| 版本锚点 | `main@8147538b4226ae41e2487a9179e3bcc1f68e8554`（2026-07-31） |
| 拉取日期 | 2026-09-17 |
| 局部参考 | —（全盘参考，无局部） |
| 原始件存放位置 | `docs/design-ref/DESIGN.stripe.md`（**参考件，非项目真源**） |
| 锁源 | `design-lock.json`（仓库根，纳入 Git） |

## §2.2 本地化调整清单

> 未列入本表的一律照参考件执行；实现与参考件不一致之处必须能在本表找到。

| # | 调整项 | 参考件原值 | 本项目值 | 理由 |
|---|--------|-----------|---------|------|
| 1 | 显示字体 | Sohne（商业授权） | Inter（next/font 自托管，无运行时外部请求）+ 中文回退 PingFang SC / Microsoft YaHei / Noto Sans SC | 拿不到 Sohne 授权；基线 §1.1#1 需实例确认项 |
| 2 | `ss01` 群集 | 全局启用 | 不启用 | Inter 无 Sohne 的 ss01 集合，不伪造；`tnum` 保留 |
| 3 | 显示/正文字重 | 300 | 400（显示与正文档） | 界面主体为中文；基线 §1.1#4：Windows 无 PingFang 细档、微软雅黑细体偏糊。拉丁与数字的编辑感靠负字距与字号对比维持 |
| 4 | display 负字距 | -1.4px ~ -0.2px | 中文显示档收敛为 0；`body-tabular`（数字）保留 -0.42px | 负字距为拉丁排版特征，中文收敛防拥挤 |
| 5 | micro-cap 字号 | 10px | 11px | 中文标签 10px 可读性不足；与 `micro` 档一致（P03 声明条 RULE-020 的 11px 下限不受影响） |
| 6 | 语义色 | 参考件无语义色板 | success `#357A5F` / warning `#B3831F`（仅图标/边/底，不作正文小字——白底 3.38:1）/ danger `#C0553F` / danger-strong `#C0553F`；暗 `#4FA98A` / `#D9A952` / `#D4694F` / `#B84E36`（对比度实测定值，2026-09-17） | 参考件明确语义色属于产品 UI 而非营销体系；原现值 6 项对比度不达标，实测后调深 |
| 7 | 涨跌色 | 参考件无 | 红涨绿跌：亮 `#C7453C/#27835A`、暗 `#DD5A50/#3EA46B`（实测调深：down 白底 4.25、暗 up 卡片底 4.46 均不过 4.5 线） | 中国习惯 + 域约定（旧基线红线 8），带 +/- 符号不裸靠颜色 |
| 8 | 图表色板 | 参考件仅给点缀色（ruby/magenta/lemon/primary-soft 作 chart 点缀） | 亮：`primary/primary-soft/ruby/lemon/ink-mute/brand-dark-900`；暗：`primary-soft/primary-subdued/magenta/ruby/lemon/ink-mute`（全部取自参考件色板内值） | 旧基线「图表固定色板、禁止随机生成颜色」规则保留；暗色顺序调亮色优先 |
| 9 | 暗色模式 | 无 | 本地派生全套：深靛调（页面 `#0E1020` / 卡片 `#151832` 系），正文对比度 ≥4.5:1 实测；暗色阴影沿用「边框替代」规则 | 基线 §1.1 无此项、旧产品已有三态机制不砍 |
| 10 | canvas 分档 | 单一 `canvas` | 亮色 `canvas` = `canvas-card` = `#FFFFFF`（照参考）；暗色拆两档（页面/卡片）供层叠 | 暗色下面板需要分层；亮色与参考零偏离 |
| 11 | mesh 适用范围 | 每个营销 hero 不可省略 | 仅 P01 首页 hero（inline SVG blob，非 CSS 渐变）；P03/P04 不加 | 基线 §1.1#3：问卷/方案是「让人干活」页，密度优先 |
| 12 | 触控目标 | 移动 ≥40px（按钮 44px） | 统一 ≥44px | 从严 |
| 13 | 卡片内距 | feature 卡 32px / dashboard 24px | 产品卡片 24px（dashboard 档） | 本产品页面是数据/操作型，密度取产品区段 |
| 14 | token 命名 | 参考件命名 | 采纳参考件命名；仓库特有语义保留原名（`up/down/chart-*/bg-overlay/micro`）；`canvas-card`、`primary-subdued-hover` 为参考件值的再命名档 | Q8 决策：后续新功能零映射 |
| 15 | 间距机制 | 8 档（2~64px） | 采纳刻度；工具类保留 `base-*` 前缀机制（防撞 Tailwind 内置刻度，见 globals.css 注释） | 机制性适配，非视觉偏离 |
| 16 | 辅助文字用色 | — | `canvas-soft` 底上的辅助文字一律 `ink-mute-2`（4.69:1）；`ink-mute` 只上 canvas/canvas-card（4.75:1，对 canvas-soft 仅 4.49 不过线） | 对比度实测结论，使用规则 |

## §2.3 断源状态

| 项 | 填写 |
|----|------|
| 断源状态 | **已断源**（改造件 `DESIGN.md` 生成后即断） |

- 禁止对根目录 `DESIGN.md` 执行 `designmd.sh update`（会从源重拉覆盖本地改造）；
- 参考件 `docs/design-ref/DESIGN.stripe.md` 保持拉取时原样，作为比对基准；
- `design-lock.json` 纳入 Git。

---

## 流程节（**待苑问改定** —— 起草自退役基线 §12/§13/§14，已适配参考件体系；按治理基线 §0.3，本节内容最终由苑问给出，AI 起草仅供改定）

### 验收 Checklist（起草）

**视觉一致性**
- [ ] 所有颜色/字号/间距/圆角/阴影取自改造件 `DESIGN.md`，与参考件的每一处不一致均能在 §2.2 找到
- [ ] 一屏最多一个实心靛蓝按钮（参考件 Do：每区块一个 filled pill）
- [ ] 按钮、标签 pill 化；卡片 12px 圆角 + hairline 边
- [ ] 金额/数值单元格 `tabular-nums`
- [ ] 涨红跌绿 + +/- 符号
- [ ] 暗色模式全 token 映射，正文对比度 ≥4.5:1 实测记录

**交互完整性**
- [ ] 可交互组件五态齐全（默认/悬停按下/禁用/加载/结果反馈）
- [ ] 空态三要素（图标+说明+引导操作）、加载骨架、错误态不回归（P01 三形态、P03 断点续答、P04 失败重试）
- [ ] 触控目标 ≥44px；动效尊重 `prefers-reduced-motion`

**可访问性**
- [ ] 正文对比度 4.5:1 / 大字 3:1（亮暗两态实测）
- [ ] 焦点态可见；语义标签不回归

**门禁**
- [ ] `scripts/check.sh` 全绿（clippy / cargo test / biome / tsc / Token 门禁 / designmd validate）

### 红线清单（起草，承旧基线红线、按新体系改写）

1. 禁止发明改造件 `DESIGN.md` 之外的颜色/字号/间距/圆角/阴影值；
2. 禁止组件中写死色值与像素（Token 门禁 RULE-006 机械化执行）；
3. 禁止涨跌色用反；禁止数值不用等宽；
4. 禁止一屏多个实心主按钮；
5. 禁止裸空态（必须有引导操作）；
6. 禁止按钮改圆角矩形（pill 是参考件铁律）；
7. 禁止对 `DESIGN.md` 执行 `designmd.sh update`；
8. 禁止把靛蓝 `primary` 用作正文文字色（CTA/链接色，参考件 Don't）。

### AI 硬约束（起草）

1. 写任何界面代码前先读根目录 `DESIGN.md`；数值一律引用其 token，发现未覆盖值→停下来问，不要自己编；
2. 本项目声明清单见 spec §2.2——那是最小偏离集，除此之外与参考件不一致即缺陷；
3. 完成后对照本 Checklist 与红线自查，逐项输出 ✅/❌。

---

## Tickets

| # | Ticket | 内容 |
|---|--------|------|
| 01 | governance-and-reference | 基线拷入、AGENTS 指针、参考件 + 锁源入库 |
| 02 | design-md-and-tokens | 改造件 `DESIGN.md` + validate 挂门禁 + `globals.css` 亮/暗重组 + Inter 接线 |
| 03 | components | ui 基元 + 布局组件重排（pill、hairline 卡、字阶改名） |
| 04 | pages | P01（含 mesh hero）/P02/P03/P04 适配 + loading/not-found |
| 05 | acceptance | 门禁全绿 + 亮/暗走查 + 走查记录 + 本 spec 流程节回填 |
