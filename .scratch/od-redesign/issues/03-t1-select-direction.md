# 03 · 对比拍板 + 胜者过 T1

Status: resolved
Labels: ready-for-agent

> 前置：02 完成。

## 步骤

1. 3 候选并排呈现，**苑问拍板**选「别扭最少」；落选 2 个丢弃（project 留存不迭代）
2. 胜者 project 成为唯一迭代载体；对话式修改补齐 T1·B 缺口（五态 / 极端数据），不整项目重生成
3. T1·A 一致性：产物 token 与声明 direction 逐项比对（可自动的自动跑）
4. T1·B 完整性六项：AI 跑自动项（自包含、对比度脚本），**苑问按表勾**中文渲染 / 五态
5. 调整清单初稿（spec §4）：AI 起草，苑问拍板

## DoD

- 拍板结论 + T1 勾选记录写入本 ticket Comments；调整清单填入 spec §4

## Comments

- 2026-09-20 · 苑问 · **拍板:GitHub 风(ww-redesign-github)**为胜者;落选 Linear / Duolingo 丢弃不迭代。胜者 project 自此为唯一迭代载体。
- 2026-09-20 · AI · **T1·A 一致性:通过**。亮色 17 个文档化 token 逐一在位比对全中(#ffffff/#f6f8fa/#1f2328/#656d76/#d0d7de/#0969da/#0550ae/#1a7f37/#9a6700/#cf222e/#1f883d/#ddf4ff/#8250df + 暗色 4 值 #0d1117/#161b22/#30363d/#e6edf3);tokens.css 合同块为 daemon 官方文件原样粘贴。中文相关的四处偏离均由 customInstructions 授权,已列 spec §4.1 调整清单初稿(字重 500 / 行高 1.75 / 标题零字距 / CJK 字体栈,另按钮微字距待拍板)。DS 暗色未覆盖区的 color-mix 派生与 GitHub 实际暗色按钮值,记 spec §4.2 派生规则,非偏离。
- 2026-09-20 · AI · **T1·B 自动项:通过**。自包含:全文件无任何外部引用(仅 svg xmlns,favicon 为 data URI);数值:tabular-nums ×4 处(等宽+右对齐);无障碍机制::focus-visible 焦点环 ×3、44px 触控目标 ×8、skip-link、role=alert、sr-only、aria-busy 骨架、prefers-reduced-motion 降级;对比度抽算:亮 muted #656d76/白 = 5.25:1,暗 muted 派生值 ≈ 6.6:1,均过 4.5:1。
- 2026-09-20 · AI · **T1·B 人眼项待苑问勾选**(预览:http://127.0.0.1:7456/api/projects/ww-redesign-github/raw/p01-home.html):① 五态观感(有方案/空态/取数失败/加载骨架四段齐全——本页无列表,"空态"与"空结果"合一);② 极端数据不破版(边界检验区:¥12,345,678.90 / −8.60% / 超长截断,375px 与桌面各看一遍);③ 中文渲染(字重/字距/行高观感);④ 暗色切换(右上角按钮,关注派生色观感);⑤ 焦点走查(Tab 一圈,焦点环可见)。勾完(或提出修改意见)即可进 ticket 04 固化。
- 2026-09-20 · 苑问 · T1·B 人眼五项全勾通过。ticket 关闭,进 ticket 04 固化。
