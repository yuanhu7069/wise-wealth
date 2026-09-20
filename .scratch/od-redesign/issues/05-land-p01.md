# 05 · P01 落地 + T2

Status: claimed
Labels: ready-for-agent

> 前置：04 完成。允许布局结构与组件调整（spec §2）；接口、数据流、业务字段不动。

## 步骤

1. 按固化产物实现 P01（三形态：有方案摘要 / 空态 / 取数失败重试，皆按新视觉）
2. 组件层跟随产物调整（ui/ 基件、mesh-backdrop 等按需重排）；shadcn 变量映射同步
3. 数值展示 / 中文渲染 / 无障碍按产物 + 调整清单执行
4. T2：与固化产物逐项比对，偏离必须在 spec §4 清单内
5. `bash scripts/check.sh` 全绿 + 亮暗双主题 + 375px 走查

## DoD

- T2 通过，check.sh 全绿，快照与实现一致（除清单内偏离）

## Comments

- 2026-09-20 · AI · **落地策略**:token 名全数保留、值整体切 GitHub 契约值——P02–P04 即刻随类名换肤,结构适配留给 06–08;避免"改值不改名"之外的中间态。globals.css 重写(架构/双路径/shadcn 映射原样),新增按钮专属 token 组(btn-primary/-hover/-ghost-hover/-danger-hover/-border/shadow,均出自 DS「Buttons」文档值;暗色 #238636/#2ea043/#21262d/#da3633 为 §4.2 GitHub 实际值)、danger-subtle/-border 错误卡组;chart 六色按语义族机械映射。
- 2026-09-20 · AI · **P01 结构落地**:layout(去 Inter webfont,system 栈)/ page(去 MeshBackdrop,hero 素带化)/ mesh-backdrop 删除 / home-view 三形态按产物重排(卡片 hairline 6px 平面、徽章 subtle pill、大数字 28 像素等宽右对齐、应急金圆点行、空态图标圆、错误卡 danger-subtle + 描边 + 重试)/ button 四变体对齐 GitHub「Buttons」/ card 去 shadow 归 6px / badge 600 字重 / topbar sticky + 品牌方标 + ghost 登出 / theme-toggle ghost 化(三态行为不变)/ footer 内容列对齐 max-w-xl + 版本等宽。
- 2026-09-20 · AI · **T2 代码级比对:通过**。结构与产物逐段对应(顶栏/hero/形态 A/B/C/骨架/页脚),偏离全部在 spec §4.1 清单(#6 内距近似、#7 图标库、#8 骨架增强)。静态值接动态见 §4.3(版本号、健康点已接)。`bash scripts/check.sh` 全绿(clippy/94 测试/biome/tsc/Token 门禁)。
- 2026-09-20 · AI · **待苑问走查**(T2 人眼):起本地环境看 P01 三形态 + 亮暗双主题 + 375px,重点①大数字等宽右对齐观感 ②错误态 subtle 底+描边 ③按钮绿(主)与灰(ghost)层级 ④暗色派生色。走查过 → 本 ticket 关闭进 06。
