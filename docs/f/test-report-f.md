# 测试报告 · 智策理财 F 期(模式库补齐与模式库页)

> 对应需求:REQ-20260925-01(prd-f.md)· 执行期:2026-09-25(单日,票 01-06)
> 验收口径:prd-f 元信息块 DoD 六条;红线来源:基线-arch §18(19 条)
> 本报告状态:**技术验收全绿**;苑问人工走查与 DoD 验收动作待执行(见 §9)

---

## 1. 自动门禁与红线

`bash scripts/check.sh` **5/5 全绿**(收口时点复跑):

| # | 门禁 | 结果 |
|---|------|------|
| 1 | cargo fmt + clippy(--all-targets) | ✅ 零警告 |
| 2 | cargo test(144 用例) | ✅ 144 passed / 0 failed |
| 3 | biome check(src + scripts) | ✅ |
| 4 | tsc --noEmit | ✅ |
| 5 | Token 门禁(check-tokens.sh) | ✅ src 无裸色值/裸 px |

**两条期级声明的 diff 级核对**:
- **零数据库迁移**:本期无 0008 迁移文件;plans/snapshots 表结构零变更(ADR-F-002 读取时解析)✅
- **零引擎改动**:`engine.rs` diff 零删除行(仅新增测试);`ShareType`/`RuleKind`/求解序未触碰(RULE-036 红线)✅

## 2. 测试构成

**域/加载层金例单测**(新增 11 用例,`cargo test` 内):

| 用例 | 断言 |
|------|------|
| 内嵌配置装载 | 4 个 L1 模式(Phase 1 目标 5,金字塔挂账)+ 全部 source 非空 |
| 标准普尔形状 | disputed 徽章、无余量/规则桶、investable=growth、必要桶=spending、出处含「从未发布」、桶 id 序列锁定 |
| 四笔钱形状 | verified、规则桶=safeguard、余量+投资=long_term、必要桶=liquid、出处含「且慢」、桶 id 零跨模式重名 |
| snp 求解 | 12,400 收入 → 1,240/2,480/3,720/4,960;达标态(monthly_toward=0);FixedExceedsNecessary 如实触发 |
| four_pots 求解 | 450,000/130,000/240,000/380,000;应急节奏 ¥1,300/月(百元粒度取整)、months_to_fill=23 |
| source fail-fast | 缺失/空白出处 → 装载失败(RULE-037) |
| share_desc | 四种 share 类型字符串化(含 12.5% 小数档) |
| entry 枚举 | 缺省 questionnaire / 两枚举可解析 / 未知值拒绝 |
| disputed 不主推 | 薄厚两档案 is_recommended 永不指向存疑模式(RULE-033) |
| 埋点载荷 | plan_generated 带 entry(两种取值),键白名单 +entry |
| 缺出处/空出处 | 分别命中两条独立错误信息 |

**service/API 集成**:无 Cookie ×2 → 401;entry=elsewhere → 422;PUT 既有链路回归全绿。

## 3. 设计闸门(T0 → T1 → T2)

| 闸门 | 结果 |
|------|------|
| **T0 参数四件套** | ✅ github / redesign-existing-projects / high-fidelity / customInstructions @ af2d3a9;daemon 连通 7456→200;载体复用 `ww-redesign-github` |
| **生成** | run `5a1a1aca`,4.9 min,stable-prompt 缓存命中 87%,**迭代轮次 0(一次生成)**;requestId `2d4d6d1c` |
| **T1·A 一致性** | ✅ 绑定契约 **63 项逐值全中**(基准 = p05 固化快照);全文 hex **超出冻结契约 0 个** |
| **T1·B 完整性** | ✅ 自动项 23/23:自包含零外链 / tabular / 焦点环 / 44px / reduced-motion / aria-expanded+hidden 收展双态 / role=status·alert / aria-busy / 徽章文字+色双通道 / 辟谣文案 / 四卡四 CTA / 免责页脚;**人眼项(亮/暗 × 375px 逐态)交苑问**(按 E 期 P02-P04 先例并入期末复核,本次已由截图预走查) |
| **T2 落地验收** | ✅ 调整清单 4 项(spec.md §4.1),**清单外零偏离** |

固化三件齐:快照 `docs/design/f-modes/p06-modes-github.html`(860 行,sha256 前 16 位 `63f852b07dafaaa6`)+ `生成记录.md` 补行 + globals.css 零改动(零新 token)。

## 4. 实机走查(生产模式 `next build && next start`;截图存 `.scratch/f-modes/shots/`)

> **环境备注**:turbopack dev 模式在 headless playwright 下 hydration 不发生(E 期 /tracking 同症状,非本期代码回归);故交互走查全部在生产模式完成,真实桌面浏览器不受影响(E 期人工走查佐证)。前端已恢复 dev 模式驻留。

| AC | 场景 | 结果 | 证据 |
|----|------|------|------|
| AC-1 | P06 列表 4 卡 + page_view p06 | ✅ | p06-desktop-light.png;埋点 §5 |
| AC-2 | 存疑卡展开:辟谣 + 四桶口径 + CTA 可用 | ✅ | p06-desktop-light-expanded.png |
| AC-3 | 首页 / 方案页双入口一步达 | ✅ | p01-with-modes-entry.png |
| AC-4 | CTA 全链路:四笔钱生成 → v12 → 方案页「第 12 版」 | ✅ | p04-after-mode-lib-generate.png;埋点 §5 |
| AC-5 | verified 方案无提示条 | ✅ | p04-verified-no-notice.png |
| AC-6 | CTA 失败(实机杀后端)→ 卡内错误条 + 重试 | ✅ | p06-cta-failure.png |
| AC-7 | 会话过期(实机清 cookie)→ 内联过期错误 + 重登链接(from=/modes) | ✅ | p06-session-expired.png;呈现沿 P03 先例,prd-f AC-6/7 措辞已回填 |
| AC-8 | 问卷步 6 存疑卡永不带「推荐」 | ✅ | recommend 金例锁定 + RULE-006 两分支走查(推荐落在 50/30/20,真实档案) |
| AC-9 | 手动生成存疑方案 → 警示条 + 方案完整可用 | ✅ | p04-disputed-warn-notice.png(临时 v13,测后恢复 v8) |
| AC-10 | caution → info 变体 | ⚠️ 验证缺口 | 库内无 caution 模式(金字塔挂账);映射纯函数 + info 蓝款复用,首个 caution 模式入库时随走查覆盖 |
| AC-11 | 库不可达错误态 | ⚠️ 架构说明 | 后端整体不可达时 requireSession 统一落登录重定向(B 期既定行为);「后端活但取数失败」场景组件与 E 期 AC-16 同构,实机模拟方式待苑问商定 |
| AC-12 | 375px 极端数据不破版 | ✅ | p06-mobile-light/dark.png |
| AC-13 | 防御空态 | ✅(代码核对) | 构建期兜底使其实际不可达;page.tsx 防御分支就位 |

五态矩阵对照 prd-f §8.3:骨架(loading.tsx)/错误/空态/CTA 生成中/正常 全部落地 ✅。

## 5. 埋点 SQL 查验

```
page_view      page_id=p06            ×4(2026-09-25 走查期)
plan_generated entry=mode_lib         ×2(四笔钱 v12 / 标准普尔 v13)
金额/余额入载荷                           0 条(正则复核 cents|余额|金额 = 0)
```

## 6. 备份与恢复演练

`restore-drill.sh` 复跑(2026-09-25 12:07):**12 项抽验全一致**(行数/金额合计/内容指纹,含 plans 快照指纹——同时佐证零迁移);副本库起服务 `/health db=ok` + 种子账号登录 + 读方案数据(第 8 版 four_accounts)全通。本期表结构零变更故无新增抽验项。

## 7. 安全审计

| 项 | 结果 | 处置 |
|----|------|------|
| cargo audit | RUSTSEC-2023-0071(rsa Marvin Attack,medium,无修可升) | **既有挂账**(E 期同项),非本期引入 |
| npm audit | js-yaml high ×2(传递依赖,fix available) | **存量**(本期零新增前端依赖);是否 `npm audit fix` 留苑问拍板(动 lockfile) |
| 金额脱敏 | 埋点/日志零金额 ✅ | §5 正则复核 |
| 输入面 | entry 枚举 422 / l1_mode 422(既有)/ 白名单零变更 ✅ | 集成测试 |

## 8. 已知偏离与遗留

**偏离(均在授权范围,spec.md §4.1)**:CTA 失败用卡内错误条(无 toast 组件)/ caution 徽章同用黄族 / 推荐卡默认展开 / 产物核验区不渲染。

**遗留(挂账)**:
1. **理财金字塔**:OUT-003,前置 = 引擎扩展 ADR(ADR-F-003)
2. **gen-types.sh TS7 修法**:E 期挂账延续;本期以真实 openapi.json 手工移植(含票 04 补漏一次),建议期收口拍板(换 openapi-typescript 版本 / 降级 TS / 锁 Node)
3. **AC-10 验证缺口**:待首个 caution 模式
4. **dev 模式 hydration 环境问题**:headless playwright + turbopack;真实浏览器无碍,建议苑问日常 dev 使用中留意(若有异常反馈再排查)
5. 既有挂账延续:RISK-E-1(TLS)/ I-8 / I-9 / I-12

## 9. 评审与修复记录 + DoD 判定

**执行中修复**(全部当天闭环):
- 票 01:测试暴露文档不一致——金字塔推后期内为 **4 个模式**,7 处「5→4」修正(prd/design/plan/custom-instructions/工单)
- 票 02:api-types 手工移植漏 components.PlanView 一处(缩进差异),票 04 类型检查暴露后补齐
- 票 04:state.ts 注释碎片 / biome 格式化 / Token 门禁 2 处(骨架裸 px、注释「px」字样)
- 票 06:prd-f AC-6/7 措辞按实机呈现回填(卡内错误条 / 内联过期错误——沿 E 期 P03 先例)

**DoD 判定**(prd-f 元信息块):

| 项 | 状态 |
|----|------|
| 功能:AC-1~13 | ✅ 技术全过(AC-10 验证缺口 / AC-11 架构说明,见 §4) |
| 体验:全链路 桌面+375px+亮/暗 | ✅(截图;**苑问人工走查待执行**) |
| 质量:test-report-f + 红线 + check.sh | ✅ 本报告;红线 19 条逐项 ✅(红线 1 零新依赖 / 红线 11 分页 N/A / 红线 15 零迁移 / 红线 16 BFF 未破 / 红线 17 RULE 注释落位 / 其余沿 B/E 期既证) |
| 设计:T0/T1/T2 + 固化 | ✅ §3 |
| 数据:4 模式入库 + 埋点可查 | ✅ §2/§5 |
| 运维:零迁移 + restore-drill + 一键启动 | ✅ §6 |
| 我的验收动作(苑问) | **待执行**:P06 用四笔钱真实生成一份方案 + 标准普尔存疑链路走查 |

**结论**:技术验收全绿,期进入人工验收阶段——待苑问按 §4 截图或实机走查 + 完成 DoD 验收动作后,prd-f 状态改「已完成」并做期收口提交。
