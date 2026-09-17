# 03: 组件层重排（pill、hairline 卡、字阶改名）

**What to build:** 把组件层的形态从旧风格重排到参考件形态：按钮 pill 化 + 参考 press 态、卡片 hairline 边 + 蓝调阴影 + 12px 圆角、徽章变 soft pill、字阶类名全量从旧名迁到参考件名。改的是 `web/src/components/` 下全部组件与散落的字阶/颜色类名。

**Blocked by:** 02

**Status:** done（2026-09-17）

**完成记录（2026-09-17）**：四个 ui 基元与布局组件全部重排到位。button pill 化（hover deep / active press）；card 加 hairline 边 + 12px 圆角 + L1 蓝调阴影；badge 对齐 pill-tag-soft；top-bar 登出改 pill、theme-toggle 圆形化；site-footer 换 caption 字阶。全仓类名迁移用 sed 批量 + 语境手工两轮完成，旧 token 名（`text-page-title/section-title/aux/label/data`、`bg-bg-*`、`text-text-*`、`border-divider`、`primary-light`）grep 零残留。注释里的 `Npx` 字面量被 Token 门禁扫出两轮，改"像素"写法。门禁：biome / tsc / Token 门禁全绿。

- [x] `ui/button.tsx`：圆角 pill；default = 实心靛蓝（hover `primary-deep` / active `primary-press`）；secondary = 白底靛蓝字 + 1px 靛蓝边；ghost = `text-action`；destructive 保留语义 danger；尺寸维持 h-11（移动）/h-10（桌面）≥44px；字阶 `button-md`（400）
- [x] `ui/card.tsx`：`rounded-lg`(12px) + `border-hairline` + `shadow-card`（L1 蓝调）；内距产品档 24px；禁止卡片套卡片注释保留
- [x] `ui/badge.tsx`：对齐 `pill-tag-soft`——`primary-bg-subdued-hover` 底 + `primary-deep` 字 + micro-cap 字阶（11px 档）
- [x] `ui/skeleton.tsx`：底色换 `canvas-soft` 系
- [x] 布局组件：`top-bar.tsx`（对齐 nav 形态：白底、hairline 分隔、登出为 ghost）、`site-footer.tsx` / `site-footer-shell.tsx`（footer-light：canvas 底、ink-mute caption 字）、`theme-toggle.tsx`（三态机制不动，仅皮肤类名）、`disclaimer.tsx`
- [x] 全仓 className 迁移（grep 兜底）：`text-page-title/section-title/body/aux/label/data` → `display-*/heading-*/body-md|body-lg/caption/micro-cap/body-tabular`；`bg-bg-page/bg-bg-card/bg-bg-subtle` → `bg-canvas/bg-canvas-card/bg-canvas-soft`；`text-text-title/body/aux/inverse/disabled` → `text-ink/ink-secondary/ink-mute/on-primary/ink-mute-2`；`border-border`→`border-hairline`、`bg-primary-bg`→`bg-primary-bg-subdued-hover`；数值列补 `tabular-nums`
- [x] 旧 token 名在 src/ 下零残留（grep 验证：`text-page-title|section-title|text-aux|text-data|bg-bg-|text-text-`）
- [x] Token 门禁 + biome + tsc 通过；注意代码与注释中不出现门禁正则命中的英文色词（white/black/red…）

**切分理由：** 组件层是「形态」变化的集中地，与 02（值）和 04（页面布局）分开，走查时能区分「token 接错」与「形态不像」。类名迁移横切所有文件，放这里一次做完，04 就只剩布局。
