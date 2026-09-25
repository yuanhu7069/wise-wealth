# 04 · P06 落地：组件层实现 + 五态接线 + T2 验收

Status: open
Labels: ready-for-agent

> f-plan T4。前置：03 完成（固化产物为落地真源）。规格：arch-f.md §1、§3（RULE-032/033）、§5；prd-f.md §8（页面清单/状态矩阵/错误清单）。

## 步骤

1. 新建 `web/src/app/modes/`（Server 壳取模式列表 + Client 卡片组件）；`/modes` 进 layout 未登录重定向链（`/login?from=/modes`）
2. 卡片列表对齐固化产物：名称 / tagline / 可信度徽章（文字+色，三值）/ 适合人群 pill / 推荐徽章（至多 1 张，取 `is_recommended`）/ 展开详情（出处块 + 桶概览 share_desc 行 + CTA）
3. 五态接线：列表骨架（路由级 loading）/ 防御空态「模式库暂时为空」/ 错误条 + 重试（A 期降级链）/ CTA 生成中 loading 禁重复 / 正常态
4. CTA 接 Server Action 生成链路（复用既有 generatePlanAction 模式，entry=mode_lib）；成功 → 跳 `/plan`；失败 → toast「生成没成功，再试一次」（ERR-F-01）；401 → 登录回跳（ERR-F-02）
5. 调整清单落 `.scratch/f-modes/spec.md` §4（产物原值 / 项目值 / 理由，逐项拍板）；D/E 期 §4.1 结论照抄项（中文字重 / CJK 字体栈 / lucide）不再论证
6. T2 验收：实现与固化产物的偏离全部能在调整清单找到，清单外零偏离
7. 走查 AC-1/2/6/11/12/13（桌面 + 375px × 亮/暗；截图存 `.scratch/f-modes/shots/`）

## DoD

- `check.sh` 全绿（含 check-tokens.sh Token 门禁）
- 五态齐全对照 prd-f §8.3；清单外零偏离
- AC-1/2/6/11/12/13 走查通过（截图为证）
- 金额/余额不渲染核对：P06 页面源码无金额格式化调用（桶概览只有比例与口径文案）
