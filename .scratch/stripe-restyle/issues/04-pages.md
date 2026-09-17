# 04: 四页适配（P01 含 mesh hero）

**What to build:** 四个页面按参考件排版语言重排：留白加大、字阶对比拉开、卡片节奏统一；P01 首页 hero 上渐变 mesh（inline SVG blob，非 CSS 渐变）。`loading.tsx` / `not-found.tsx` 同步换肤。

**Blocked by:** 03

**Status:** done（2026-09-17）

**完成记录（2026-09-17）**：新增 `app/mesh-backdrop.tsx`（inline SVG 五色斑、stdDeviation 90 重度重叠），P01 页首改为全宽 hero 带 + display-lg 标题；P02 输入框对齐 text-input 形态（hairline-input 边、focus 靛蓝）；P03 选中态改 `bg-primary/10`（首轮走查发现 subdued 实底在暗色下与翻白文字对比崩坏，改半透明叠底后两态自洽）；P04 五段保留、`chartBgClass` 无需改动（chart token 名未变）、金额列走 body-tabular 自动 tnum。业务逻辑零改动（纯 className 与展示 JSX）。首轮视觉评审 4 项 fail（mesh 浑浊+硬接缝、暗色选中卡、step1 截图错位、移动端 mesh）——修复后复验 9/9 pass，详见 05 走查记录。

- [x] P01 `/`（home-view）：hero 区渐变 mesh 背景（cream→orange→lavender→indigo→ruby 横向色带 + 有机 blob，SVG 实现，`aria-hidden`），标题 `display-xl/xxl`（移动 36px 档降级），主 CTA 单个实心 pill；三形态（方案摘要卡 / 空态引导 / 取数失败重试）类名与形态换新，逻辑零改动
- [x] P02 `/login`：卡片居中 + hairline + 输入框对齐 `text-input`（白底、`hairline-input` 边、focus 边变 primary）；表单逻辑与回跳白名单零改动
- [x] P03 `/questionnaire`：一屏一题版式保留；选项卡片 hairline + 选中态靛蓝；底部固定操作栏 11px 声明条（RULE-020）不回归；自动保存/断点续答/预填逻辑零改动
- [x] P04 `/plan`：五段结构保留；段落标题 `heading-*` 阶梯；CSS 条形图颜色改走 `chart-*` 新 token；风险提示用语义色；`state.ts` 的 `chartBgClass` 映射同步
- [x] `loading.tsx` 骨架屏、`not-found.tsx` 换新 token
- [x] 全部页面：移动端重排（非缩放）检查一遍 375px；金额/数字处 `tabular-nums`
- [x] 页面业务逻辑（server 取数、server action、状态机）零改动——本 ticket 只动 className 与纯展示 JSX
- [x] Token 门禁 + biome + tsc + `next build` 通过

**切分理由：** 页面是「布局与留白」的载体，与组件形态分开；四页体量小（~1700 行 tsx 中页面占大头），一个 ticket 内可完成并自洽走查。mesh 是唯一新增视觉元素，收在本 ticket 不外溢。

## Comments

**2026-09-17 · 操作栏恒钉视口底（苑问反馈"上下一步跳来跳去"）**：design-v2 v0.5 本就要求"跨步骤位置恒定"，原实现 `sticky bottom-0` 只在内容高于视口时生效，矮步骤按钮停在内容正下方。改法：问卷页外壳（main/Card/CardContent）逐层 `flex-1` 拉成全高列，wizard 根 `flex-1`，操作栏 `mt-base-md` → `mt-auto`。实测步 3（矮）与步 5 滚到底落座位置像素级一致；高步骤未滚动时钉住态比落座态高约 40px（sticky bottom-0 贴边 vs 页面底距+卡片内距），属 sticky 标准行为，维持贴边方案（备选 `bottom-base-lg` 悬浮可像素归位，代价是滚动时按钮下露 16px 内容缝，未采用）。截图：`shots/bar-step3-short.png`、`bar-step5-tall.png`、`bar-step5-scrolled.png`。
**2026-09-17 · 重新生成入口行为（苑问反馈）**：① P01/P04「重新生成」改为 `/questionnaire?restart=1`——问卷页见参落步 1（答案照常预填），未带参仍断点续答；restart 访问不计「问卷开始」埋点。② 步 6 模式装载后默认选中推荐项（函数式 setChosen 仅代填空值），「生成方案」不再因未点卡片而禁用，用户仍可改选。实测：重新生成 → 步 1 ✓；直走到步 6 → 生成方案 enabled ✓（截图 `shots/restart-step1.png`、`restart-step6.png`）。
