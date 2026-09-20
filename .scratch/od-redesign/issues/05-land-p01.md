# 05 · P01 落地 + T2

Status: needs-triage
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
