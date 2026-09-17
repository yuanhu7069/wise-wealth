---
version: alpha
name: wise-wealth-stripi-adapted
description: "Adapted from the Stripi-inspired design analysis for 智策理财 (wise-wealth), a Chinese personal fintech decision tool. Electric indigo primary on white canvas, deep navy ink, hairline cards with blue-tinted shadows, pill buttons, and a signature gradient mesh reserved for the home hero. Type is Inter (Sohne substitute) tuned for Chinese: weight 400 with zero display tracking, tabular figures for money. A full dark mode is derived locally in the navy-indigo family."

colors:
  primary: "#533afd"
  primary-deep: "#4434d4"
  primary-press: "#2e2b8c"
  primary-soft: "#665efd"
  primary-bg-subdued-hover: "#b9b9f9"
  brand-dark-900: "#1c1e54"
  ink: "#0d253d"
  ink-secondary: "#273951"
  ink-mute: "#64748d"
  ink-mute-2: "#61718a"
  on-primary: "#ffffff"
  canvas: "#ffffff"
  canvas-card: "#ffffff"
  canvas-soft: "#f6f9fc"
  canvas-cream: "#f5e9d4"
  hairline: "#e3e8ee"
  hairline-input: "#a8c3de"
  success: "#357a5f"
  warning: "#b3831f"
  danger: "#c0553f"
  danger-strong: "#c0553f"
  up: "#c7453c"
  down: "#27835a"
  chart-1: "#533afd"
  chart-2: "#665efd"
  chart-3: "#ea2261"
  chart-4: "#9b6829"
  chart-5: "#64748d"
  chart-6: "#1c1e54"
  ruby: "#ea2261"
  magenta: "#f96bee"
  lemon: "#9b6829"
  shadow-blue: "#003770"

typography:
  display-xxl:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 56px
    fontWeight: 400
    lineHeight: 1.03
    letterSpacing: 0px
  display-xl:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 48px
    fontWeight: 400
    lineHeight: 1.15
    letterSpacing: 0px
  display-lg:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 32px
    fontWeight: 400
    lineHeight: 1.1
    letterSpacing: 0px
  display-md:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 26px
    fontWeight: 400
    lineHeight: 1.12
    letterSpacing: 0px
  heading-lg:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 22px
    fontWeight: 400
    lineHeight: 1.1
    letterSpacing: 0px
  heading-md:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 20px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
  heading-sm:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 18px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
  body-lg:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
  body-md:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 15px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
  body-tabular:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: -0.42px
    fontFeature: tnum
  button-md:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.0
    letterSpacing: 0px
  button-sm:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 14px
    fontWeight: 400
    lineHeight: 1.0
    letterSpacing: 0px
  caption:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
    fontFeature: tnum
  micro:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 11px
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: 0px
  micro-cap:
    fontFamily: "inter-var, 'Inter', 'PingFang SC', 'Microsoft YaHei', 'Noto Sans SC', system-ui, sans-serif"
    fontSize: 11px
    fontWeight: 400
    lineHeight: 1.15
    letterSpacing: 0.1px

rounded:
  xs: 4px
  sm: 6px
  md: 8px
  lg: 12px
  xl: 16px
  pill: 9999px

spacing:
  xxs: 2px
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  xxl: 32px
  huge: 64px

components:
  button-primary-pill:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.button-md}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  button-primary-pill-pressed:
    backgroundColor: "{colors.primary-press}"
    textColor: "{colors.on-primary}"
    typography: "{typography.button-md}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  button-secondary:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.primary}"
    typography: "{typography.button-md}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  button-on-dark:
    backgroundColor: "{colors.brand-dark-900}"
    textColor: "{colors.on-primary}"
    typography: "{typography.button-md}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  text-input:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    padding: 8px 12px
  text-input-focused:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    padding: 8px 12px
  card-feature-light:
    backgroundColor: "{colors.canvas-card}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.lg}"
    padding: 24px
  card-pricing:
    backgroundColor: "{colors.canvas-card}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.lg}"
    padding: 32px
  card-pricing-featured:
    backgroundColor: "{colors.brand-dark-900}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.lg}"
    padding: 32px
  card-cream-band:
    backgroundColor: "{colors.canvas-cream}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.lg}"
    padding: 32px
  status-chip-success:
    backgroundColor: "{colors.success}"
    textColor: "{colors.on-primary}"
    typography: "{typography.micro-cap}"
    rounded: "{rounded.pill}"
    padding: 4px 8px
  status-chip-warning:
    backgroundColor: "{colors.warning}"
    textColor: "{colors.ink}"
    typography: "{typography.micro-cap}"
    rounded: "{rounded.pill}"
    padding: 4px 8px
  status-chip-danger:
    backgroundColor: "{colors.danger}"
    textColor: "{colors.on-primary}"
    typography: "{typography.micro-cap}"
    rounded: "{rounded.pill}"
    padding: 4px 8px
  numeric-cell-up:
    backgroundColor: "{colors.canvas-card}"
    textColor: "{colors.up}"
    typography: "{typography.body-tabular}"
    rounded: "{rounded.xs}"
    padding: 0px
  numeric-cell-down:
    backgroundColor: "{colors.canvas-card}"
    textColor: "{colors.down}"
    typography: "{typography.body-tabular}"
    rounded: "{rounded.xs}"
    padding: 0px
  pill-tag-soft:
    backgroundColor: "{colors.primary-bg-subdued-hover}"
    textColor: "{colors.primary-press}"
    typography: "{typography.micro-cap}"
    rounded: "{rounded.pill}"
    padding: 4px 8px
  nav-bar-on-mesh:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink}"
    typography: "{typography.body-md}"
    rounded: "{rounded.xs}"
    padding: 16px 24px
  link-on-light:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.primary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.xs}"
    padding: 0px
  footer-light:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.ink-mute}"
    typography: "{typography.caption}"
    rounded: "{rounded.xs}"
    padding: 64px 24px

---

## Overview

智策理财的界面语言以渐变 mesh 开场，但只在唯一一处：P01 首页 hero。一条奶油、暖橙、粉、电靛、玫红的横向柔和色带占据首页上部，是品牌的第一印象锚点。其余三页（登录 / 问卷 / 方案）是「让人干活」的产品页：白色 canvas 之上是 hairline 边框的卡片、蓝调浅阴影、pill 按钮，密度优先，不做氛围装饰。

色彩系统里 **靛蓝**（`{colors.primary}` — `#533afd`）是唯一的 CTA 色，克制使用：每个区块至多一个实心 pill。**深海军蓝**（`{colors.ink}` — `#0d253d`）是全局正文色。本产品是中文个人理财工具，涨红跌绿（`{colors.up}` / `{colors.down}`）承载域语义，是参考品牌没有的本地约定；金额数字一律 `tnum` 等宽。

排版围绕 **Inter**（Sohne 的开源替代）weight 400 展开：本项目界面主体是中文，细体在 Windows 中文字体下发糊，故显示/正文档加重一档并取消负字距（见文末 Local Adjustments #3/#4）；拉丁数字的财务感由 `tnum` 与 `body-tabular` 的 -0.42px 字距维持。暗色模式为本地派生（见 Colors §Dark mode），参考品牌无暗色体系。

**Key Characteristics:**
- Gradient-mesh backdrop on the home hero only — cream/orange/lavender/indigo/ruby washed horizontally, implemented as inline SVG blobs (not CSS gradients).
- Single-indigo CTA hierarchy: at most one filled `{colors.primary}` pill per section.
- Inter at weight 400 for display and body (Chinese-tuned); tabular figures (`tnum`) on every money cell.
- Up = red, down = green (Chinese market convention) with +/- signs — color never carries the signal alone.
- Hairline cards (`{colors.hairline}` 1px, `{rounded.lg}` 12px) on white canvas with subtle blue-tinted shadows.
- Pill buttons (`{rounded.pill}`) with tight `8px 16px` padding; ≥44px touch targets on mobile.
- Derived navy-indigo dark mode: page `#0e1020`, cards lift via hairline borders instead of shadows.

## Colors

> **Source:** Stripi-inspired analysis (home / payments / pricing / dashboard pages), adapted per Local Adjustments below.

### Brand & Accent
- **Indigo** (`{colors.primary}` — `#533afd`): 唯一 CTA 色。实心 pill 按钮、链接强调、mesh 锚点。禁止用作正文文字色。
- **Indigo Deep** (`{colors.primary-deep}` — `#4434d4`): hover 态 / 渐变中间站 / soft pill 前景。
- **Indigo Press** (`{colors.primary-press}` — `#2e2b8c`): 按下态。
- **Indigo Soft** (`{colors.primary-soft}` — `#665efd`): 亮色图表第二色、暗色图表首色。
- **Indigo Subdued** (`{colors.primary-bg-subdued-hover}` — `#b9b9f9`): soft 标签底、悬停底、暗色下的可点击文字色。
- **Brand Dark 900** (`{colors.brand-dark-900}` — `#1c1e54`): 深色面板 / on-dark 按钮 / 亮色图表末档。
- **Ruby** (`{colors.ruby}` — `#ea2261`): mesh 与图表点缀；**不作按钮色**。
- **Magenta** (`{colors.magenta}` — `#f96bee`): mesh 色站、暗色图表点缀。
- **Lemon** (`{colors.lemon}` — `#9b6829`): mesh 暖色站、图表暖档。

### Surface
- **Canvas** (`{colors.canvas}` — `#ffffff`): 页面底。
- **Canvas Card** (`{colors.canvas-card}` — `#ffffff`): 卡片底。亮色与 canvas 同值（照参考）；暗色拆档见 Dark mode。
- **Canvas Soft** (`{colors.canvas-soft}` — `#f6f9fc`): 冷调次级面（分区带、骨架屏）。
- **Canvas Cream** (`{colors.canvas-cream}` — `#f5e9d4`): 暖色间歇带（仅首页）。
- **Hairline** (`{colors.hairline}` — `#e3e8ee`): 卡片与分隔线 1px 边。
- **Hairline Input** (`{colors.hairline-input}` — `#a8c3de`): 输入框边。

### Text
- **Ink** (`{colors.ink}` — `#0d253d`): 标题与强调正文。深海军蓝，非纯黑。
- **Ink Secondary** (`{colors.ink-secondary}` — `#273951`): 默认正文。
- **Ink Mute** (`{colors.ink-mute}` — `#64748d`): 辅助文字、caption、表格标签。
- **Ink Mute 2** (`{colors.ink-mute-2}` — `#61718a`): 禁用文字（本地指派，参考件无 disabled 档）。
- **On Primary** (`{colors.on-primary}` — `#ffffff`): 靛蓝/深蓝面板上的前景。

### Semantic（本地新增，参考件无语义色板）
- **Success** (`{colors.success}` — `#357a5f`) / **Warning** (`{colors.warning}` — `#b3831f`) / **Danger** (`{colors.danger}` — `#c0553f`): 产品 UI 状态色（对比度实测调深）。
- **Danger Strong** (`{colors.danger-strong}` — `#c0553f`): 危险按钮专用底（白字达标）；暗色取深档，与 danger 文字档分离。
- **Warning 使用规则**：warning 对白底只有 3.38:1——**不作正文小字**；文本提示用 ink/danger/success 表达，warning 只用于图标、边框、底色（UI 元素 ≥3:1 达标）。
- **Up** (`{colors.up}` — `#c7453c`) / **Down** (`{colors.down}` — `#27835a`): 红涨绿跌（中国习惯 + 域约定），必须带 +/- 符号。

### Chart（固定六色，禁止随机取色）
亮色取值顺序 `{colors.chart-1..6}`（首色即主色）；暗色顺序为 primary-soft / subdued / magenta / ruby / lemon / ink-mute（全部取自参考色板，见 Dark mode）。

### Dark mode（本地派生）
参考品牌无暗色体系。本项目按深靛调派生全套，正文对比度 ≥4.5:1 实测：

| Token | Dark value |
|---|---|
| canvas | `#0e1020` |
| canvas-card | `#151832` |
| canvas-soft | `#1b1f3b` |
| canvas-cream | `#2a2440`（暖带取薰衣草深调） |
| hairline | `#262b4d` |
| hairline-input | `#3f4670` |
| ink | `#eef0fb` |
| ink-secondary | `#c3c8de` |
| ink-mute | `#848cad` |
| ink-mute-2 | `#6b7294` |
| primary / deep / press / on-primary | 不变（白字对 `#533afd` 6.2:1） |
| primary-bg-subdued-hover | `#b9b9f9`（兼作暗色可点击文字色，对卡片底 ≥9:1） |
| brand-dark-900 面板 | 暗色下以 `primary` 代偿（深底上再叠深蓝不可辨） |
| success / warning / danger | `#4fa98a` / `#d9a952` / `#d4694f` |
| danger-strong | `#b84e36`（白字 5.02:1；danger #d4694f 作按钮底只有 3.54） |
| up / down | `#dd5a50`（4.68:1，#d9564c 只有 4.46）/ `#3ea46b` |
| chart-1..6 | `#665efd` / `#b9b9f9` / `#f96bee` / `#ea2261` / `#9b6829` / `#848cad` |
| 阴影 | 一律 `0 0 0 1px {colors.hairline暗值}`（边框替代，降低阴影存在感） |
| 遮罩 | `rgba(4,6,16,0.6)` |

三态机制：跟随系统（media）+ 手动 `.dark` / `.light` class，暗色值单点定义、机械映射，禁止两处书写。

## Typography

### Font Family

显示与 UI 层为 **Inter**（`next/font` 自托管，权重 400/500/600；无运行时外部请求），中文回退 PingFang SC → Microsoft YaHei → Noto Sans SC。`tnum`（等宽数字）用于一切金额/数值单元格；Sohne 的 `ss01` 风格集在 Inter 上不可用，不启用（Local Adjustments #2）。

### Hierarchy

| Token | Size | Weight | Line Height | Letter Spacing | Use |
|---|---|---|---|---|---|
| `{typography.display-xxl}` | 56px | 400 | 1.03 | 0 | 首页 hero 主标题（移动降 36px） |
| `{typography.display-xl}` | 48px | 400 | 1.15 | 0 | 区块 opener |
| `{typography.display-lg}` | 32px | 400 | 1.1 | 0 | 卡片标题 / 子区块 |
| `{typography.display-md}` | 26px | 400 | 1.12 | 0 | 内页页标题 |
| `{typography.heading-lg}` | 22px | 400 | 1.1 | 0 | 大卡标题 |
| `{typography.heading-md}` | 20px | 400 | 1.4 | 0 | 段落子标题 |
| `{typography.heading-sm}` | 18px | 400 | 1.4 | 0 | 小节标题 |
| `{typography.body-lg}` | 16px | 400 | 1.4 | 0 | 首页导语 |
| `{typography.body-md}` | 15px | 400 | 1.4 | 0 | 默认正文 |
| `{typography.body-tabular}` | 14px | 400 | 1.4 | -0.42px | 金额/数字（`tnum`） |
| `{typography.button-md}` | 16px | 400 | 1.0 | 0 | pill 按钮标签 |
| `{typography.button-sm}` | 14px | 400 | 1.0 | 0 | 紧凑 pill 标签 |
| `{typography.caption}` | 13px | 400 | 1.4 | 0 | 辅助说明、表格标签（`tnum` 适用时） |
| `{typography.micro}` | 11px | 400 | 1.4 | 0 | 声明条等小字（P03 操作栏 RULE-020 下限） |
| `{typography.micro-cap}` | 11px | 400 | 1.15 | +0.1px | 标签/眉标（10px→11px，中文可读性） |

### Principles
- **层级靠字号与留白，不靠加粗**：正文一律 400；需要强调时用 `ink` 色与字号档差，不引入 600+ 的通用正文加粗（按钮/标签亦 400）。
- **金额必须 `tnum`**：任何渲染金额、笔数、比例的单元格用 `body-tabular`。
- **中文优先调优**：字重 400、display 零负字距（Local Adjustments #3/#4）；数字列保留负字距与等宽。

## Layout

### Spacing System
- **基准 8px**，细档 2 / 4 / 12。
- **Tokens**: `{spacing.xxs}` 2px · `{spacing.xs}` 4px · `{spacing.sm}` 8px · `{spacing.md}` 12px · `{spacing.lg}` 16px · `{spacing.xl}` 24px · `{spacing.xxl}` 32px · `{spacing.huge}` 64px。
- 首页营销区段 padding 64px 起；产品页（问卷/方案）32–48px。
- 产品卡片内距 24px（dashboard 档，Local Adjustments #13）。

### Grid & Container
- 首页 ~1200px 居中容器，mesh 满幅；内容页 640–760px 窄容器（问卷/方案为阅读与操作流）。
- 产品卡片网格 3→2→1 列（1024 / 768 断点）。

### Whitespace Philosophy
产品页留白比营销页收敛但显著大于旧版：段落间距 32px 起，卡片间 24px；hero 区（仅首页）上三分之一为 mesh，其下白色 canvas 宽绰留白。

## Elevation & Depth

| Level | Treatment | Use |
|---|---|---|
| 0 | Flat | 默认面 |
| 1 | `box-shadow: rgba(0,55,112,0.08) 0 1px 3px` | 白底卡片 |
| 2 | `box-shadow: rgba(0,55,112,0.08) 0 8px 24px, rgba(0,55,112,0.04) 0 2px 6px` | 悬浮面板 |
| 3 | 渐变 mesh 背景 | 氛围色而非字面阴影（仅首页 hero） |
| modal | `box-shadow: rgba(0,55,112,0.16) 0 16px 48px, rgba(0,55,112,0.08) 0 4px 12px`（L2 加深派生档） | 遮罩弹层 |

暗色下所有阴影一律替换为 `0 0 0 1px` hairline 暗值的边框表达。

### Decorative Depth
mesh 是唯一的装饰性深度语言：inline SVG 有机 blob 叠加实现，**不用 CSS 线性渐变模拟**；仅出现在 P01 首页 hero，`aria-hidden`。

## Shapes

### Border Radius Scale

| Token | Value | Use |
|---|---|---|
| `{rounded.xs}` | 4px | hairline 标签、表格 chrome |
| `{rounded.sm}` | 6px | 表单输入 |
| `{rounded.md}` | 8px | 紧凑卡、提醒条 |
| `{rounded.lg}` | 12px | 产品卡、feature 卡 |
| `{rounded.xl}` | 16px | 面板级容器 |
| `{rounded.pill}` | 9999px | 全部按钮、标签 pill |

## Components

### Buttons

**`button-primary-pill`** — 全局主导 CTA。
- 底 `{colors.primary}`、字 `{colors.on-primary}`、型 `{typography.button-md}`、padding 8px 16px、pill。
- hover 底 `{colors.primary-deep}`；按下 `button-primary-pill-pressed` 底 `{colors.primary-press}`。
- 每区块至多一个实心靛蓝按钮。

**`button-secondary`** — 描边替代。
- 白底、靛蓝字、1px 靛蓝边，同 pill 几何。

**`button-on-dark`** — mesh hero / 深色面板上用。
- 底 `{colors.brand-dark-900}`、字 `{colors.on-primary}`，同 pill 几何。

**危险操作**：底 `{colors.danger-strong}`、字白（语义色按钮，本地约定；参考件不设红按钮）。

### Cards & Containers

**`card-feature-light`** — 产品卡通用形态。
- 底 `{colors.canvas-card}`、padding 24px、`{rounded.lg}` 12px、1px `{colors.hairline}` 边、可选 L1 阴影。禁止卡片套卡片。

**`card-pricing` / featured / cream-band**（参考件保留定义，本项目暂无定价场景）：底 `{colors.canvas-card}` / `{colors.brand-dark-900}` / `{colors.canvas-cream}`，padding 32px，`{rounded.lg}`。未来新增定价/对比页时启用。

**`text-input`** — 表单字段。
- 白底、`{colors.ink}` 字、padding 8px 12px、`{rounded.sm}` 6px、1px `{colors.hairline-input}` 边；focus 边变 `{colors.primary}`。输入框高度 ≥40px。

### Navigation

**`nav-bar-on-mesh`** — 顶栏。
- 白底（滚动后保持白底）、`{colors.ink}` 字、padding 16px 24px；左标识、右登录/登出 + 主 CTA。产品页顶栏同型、hairline 下边。

### Pills, Tags, and Chips

**`pill-tag-soft`** — 软标签。
- 底 `{colors.primary-bg-subdued-hover}`、字 `{colors.primary-press}`、型 `{typography.micro-cap}`（11px）、padding 4px 8px、pill。（前景取 press 而非 deep：deep 对 subdued 底只有 4.22:1，不过 11px 中文标签的 4.5 线；press 6.17:1。）

### Signature Components

**Gradient Mesh Backdrop** — 横向柔和色带 + 有机 blob，色站只取已 token 化的五值：`{colors.canvas-cream}`（奶油）→ `{colors.lemon}`（暖橙褐）→ `{colors.magenta}`（粉）→ `{colors.primary}`（靛蓝）→ `{colors.ruby}`（玫红）；大半径模糊 + 低透明度叠出 pastel 感；inline SVG 实现；**仅 P01 首页 hero**（Local Adjustments #11）。

**Tabular-Figure Money Type** — 一切金额/数字用 `tnum`；红涨绿跌带 +/- 符号。

**`link-on-light`** — 亮底行内链接：`{colors.primary}` 字、默认无下划线、hover 下划线。

**`footer-light`** — 站点页脚：canvas 底、`{colors.ink-mute}` 字、`{typography.caption}`、padding 64px 24px。

## Do's and Don'ts

### Do
- `{colors.primary}` 只用于实心 CTA 与链接强调，每区块至多一个实心按钮。
- 首页 hero 上 mesh；产品页保持密度优先、不加氛围装饰。
- 显示与正文档一律 weight 400，层级靠字号与留白。
- 一切金额/数值单元格 `tnum`。
- 涨红跌绿且带 +/- 符号。

### Don't
- 不要把靛蓝 `primary` 当正文文字色（CTA/链接专用）。
- 不要给按钮换圆角矩形（pill 是铁律）。
- 不要在参考色板外发明新色（含 mesh 色站——色站只取上文五个 token 值）；语义色/涨跌色/图表色只用本文档已定义值。
- 不要缩短按钮 padding 至 8px 16px 以下。
- 不要渲染金额时漏 `tnum`。
- 不要用 CSS 线性渐变模拟 mesh。

## Responsive Behavior

### Breakpoints

| Name | Width | Key Changes |
|---|---|---|
| Wide | ≥ 1440px | mesh 满幅；内容容器 1200px |
| Desktop | 1024–1440px | 默认 |
| Tablet | 768–1023px | 卡片 2 列；display 降档 |
| Mobile | < 768px | 单列；display 56→36px；操作栏固定底部 |

### Touch Targets
- 按钮/可交互目标 ≥44×44px（从严于参考的 40px）。
- 输入框高度 ≥40px。

### Collapsing Strategy
- Display 阶梯 56 → 48 → 32 → 26px 随断点下降。
- mesh 在移动端收窄高度但不清失。
- 问卷操作栏在 375px 保持单行（micro 11px 声明条）。

## Iteration Guide

1. 一次只改一个组件。
2. 直接引用 token 名（`{colors.primary}`、`{rounded.pill}`）。
3. 改完跑 `npx designmd.sh validate ./DESIGN.md`。
4. 新变体单独立目。
5. 正文默认 `{typography.body-md}`（15px）；金额/数字一律 `{typography.body-tabular}`。
6. mesh 仅首页；产品页不加。

---

## Local Adjustments（本地化调整清单 · 精简版）

> 本文档为改造件（已断源，禁止 `designmd.sh update`）。参考件见 `docs/design-ref/DESIGN.stripe.md`。
> 完整版清单（含理由）见 `.scratch/stripe-restyle/spec.md` §2.2。**未列于此的一律照参考件执行。**

| # | 调整项 | 参考件 | 本项目 |
|---|--------|--------|--------|
| 1 | 字体 | Sohne | Inter（next/font 自托管）+ PingFang SC / Microsoft YaHei / Noto Sans SC |
| 2 | `ss01` | 全局启用 | 不启用（Inter 无此集）；`tnum` 保留 |
| 3 | 显示/正文字重 | 300 | 400（中文调优） |
| 4 | display 负字距 | -1.4 ~ -0.2px | 中文档 0；`body-tabular` 保 -0.42px |
| 5 | micro-cap | 10px | 11px |
| 6 | 语义色 | 无 | success `#357A5F` / warning `#B3831F`（仅图标/边/底，不作小字）/ danger `#C0553F` / danger-strong 按钮底档 |
| 7 | 涨跌色 | 无 | 红涨绿跌（亮 `#C7453C/#27835A`、暗 `#DD5A50/#3EA46B`，实测定值）+ 符号 |
| 8 | 图表六色 | 仅点缀色 | 亮/暗两组固定顺序，值全部取自参考色板 |
| 9 | 暗色模式 | 无 | 本地派生深靛调全套（见 Colors §Dark mode） |
| 10 | canvas | 单值 | 亮色单值照参考；暗色拆 canvas/canvas-card 两档 |
| 11 | mesh | 每个营销 hero | 仅首页 hero |
| 12 | 触控 | ≥40px（按钮 44） | 统一 ≥44px |
| 13 | 卡片内距 | feature 32 / dashboard 24 | 产品卡取 24px 档 |
| 14 | 危险按钮 | 无 | danger-strong 底白字（语义色按钮） |
