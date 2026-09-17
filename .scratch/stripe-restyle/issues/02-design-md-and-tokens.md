# 02: 改造件 DESIGN.md + globals.css 亮/暗重组 + Inter 接线

**What to build:** 以参考件为底产出根目录改造件 `DESIGN.md`（frontmatter token + 八段正文 + 文末精简版调整清单），把 spec §2.2 的 15 条本地化调整落成具体值；`designmd.sh validate` 通过并挂入 `scripts/check.sh`；`web/src/app/globals.css` 逐 Token 抄入改造件（亮色照参考、暗色按 §2.2#9 派生），token 命名换成参考件命名；`layout.tsx` 接入 next/font Inter。

**Blocked by:** 01

**Status:** done（2026-09-17）

**完成记录（2026-09-17）**：改造件 `DESIGN.md` 产出并 validate PASS（19 组件；剩余警告为调色板 token 未被组件引用，与参考件同类，门禁以 exit 0 为准）。对比度实测（`tmp-contrast-check.py`）首跑抓出 6 项不达标，全部修复：down 亮色调深 `#27835A`（原 4.25）、up 暗色调亮 `#DD5A50`（原 4.46）、success 亮色调深 `#357A5F`（原 3.96）、warning 亮色 `#B3831F` 并定「仅图标/边/底」使用规则（原 2.66）、新增 `danger-strong` 按钮底档（暗 `#B84E36`，原白字对 danger 3.54）、canvas-soft 底辅助文字改用 ink-mute-2（规则入 spec §2.2#16）。值已同步 DESIGN.md / globals.css / spec。mesh 一度引入两个参考件没有的色站（自纠：全部改用已 token 化五值，柔和感靠透明度+模糊）。check.sh 新增第 6 步 designmd validate。layout.tsx 接入 next/font Inter（`--font-inter` 变量）。`npm run build` 另行验证（见 05）。门禁：biome / tsc / Token 门禁 / designmd validate 全绿。

- [x] 改造件 `DESIGN.md`：frontmatter + body 八段 + 文末 Local Adjustments 精简清单
- [x] `npx designmd.sh validate ./DESIGN.md` 通过（PASS）
- [x] `scripts/check.sh` 增加第 6 步：designmd validate
- [x] `globals.css` 重写：亮色照抄改造件、暗色 `--dark-*` 单点派生、两条生效路径机械映射保留、shadcn 别名层重接
- [x] token 命名迁移至参考件命名（含 chart/mesh 规则、base-* 间距机制保留）
- [x] 暗色派生值对比度实测 ≥4.5:1（6 项修复，脚本与结果留存）
- [x] 阴影：亮色 L1/L2 蓝调 + modal 派生档；暗色「边框替代」
- [x] `layout.tsx`：next/font Inter 接入，三态主题机制与防闪烁脚本不动
- [x] `globals.css` 头注释改指改造件为唯一值来源
- [x] biome + tsc + Token 门禁通过

**实现期补充约定**：① `--text-body-tabular` 工具类经 globals 增强规则自动附带 `tnum`；② 状态 chip 三色 + 涨跌数字格已定义进 DESIGN.md components（status-chip-* / numeric-cell-*）；③ warning 亮色不作正文小字（3.38:1），文本提示用 ink/danger/success。

**切分理由：** 改造件与 token 层是纯值替换，不碰组件形态；与 03 的形态重排分开，任何一步门禁红都能定位是「值接错」还是「形态改坏」。暗色派生只在这一处发生，单点可审。
