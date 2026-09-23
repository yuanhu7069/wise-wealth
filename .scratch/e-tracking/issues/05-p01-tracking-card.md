# 05 · P01 追踪卡：摘要形态改造 + 打卡入口

Status: resolved
Labels: ready-for-agent

> e-plan T5。前置：04 完成。**代码级改动，不回炉生成**（基线：落地后真源 = 代码库）。

## 步骤

1. P01 Server 取数扩展：快照 summary（最新月是否已打卡 / 应急缺口 / persisted_months）
2. 摘要形态追加追踪卡：当月未打卡 → 「本月还没打卡」+「去打卡」主按钮（→ `/tracking`）；已打卡 → 最近快照月份 + 缺口月数摘要行
3. 空态形态（无方案）不变；卡片加载失败静默隐藏（不阻塞 P01 主体，prd-e §8.3）
4. 骨架态对齐既有 P01 骨架模式
5. `page_view p01` 既有埋点不受影响；不新增事件

## DoD

- `bash scripts/check.sh` 全绿
- 双形态 × 亮/暗 × 375px 走查过（含追踪卡各态：未打卡/已打卡/骨架/隐藏）
- 未回炉生成核对：本次无任何 OpenDesign 调用

## Comments

- 2026-09-23 · AI · 完成。page.tsx 并行追加追踪摘要取数(失败 → null → 卡片整块隐藏,不污染方案摘要);home-view.tsx 增 TrackingCard(未打卡 = 警戒点 + 已坚持月数 + 主按钮「去打卡」;已打卡 = 绿点 + 月份 + 应急金缺口一行 + ghost「去追踪」);空态/错误形态不变。实机两态验证:未打卡(本月还没打卡/去打卡)→ PUT → 已打卡(本月已打卡 2026-09 + 还差 5.6 个月 + 去追踪,且 P05 联动预填「覆盖本月」)→ DELETE 回未打卡。零 OpenDesign 调用。check.sh 全绿。
- 2026-09-23 · AI · 调试插曲备案:跨 Bash 调用的环境变量不驻留导致多次启动失败(/tmp/ww-start-server.sh 助手脚本解决);curl cookie jar 按 host 域绑定,`127.0.0.1` 与 `localhost` 不互通(验证脚本统一走 127.0.0.1)。
