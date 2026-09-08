# A 期测试报告(test-report-a.md)

> 项目:智策理财 A 期工程骨架 · 生成:2026-09-07 · 依据:arch-a.md §10/§11、基线 arch §16 审查清单、a-plan.md §4 验收标准回链
>
> 结论:**A 期验收全部通过(9/9 AC,2026-09-08 苑问验收确认)**。详见各表标注。
>
> **2026-09-08 更新**:浏览器侧校验由 Claude 完成(gstack browse 无头 Chromium 起步,升级为 playwright-core 直连 ms-playwright chromium-1208)——AC-7/AC-9 及 §5 全部 8 项已校验,**全部以真浏览器计算样式断言直接通过**(含暗色 colorScheme 双上下文、reduced-motion 双上下文);AC-8 的 T6 记录方式已更正(见 §1);遗留项 5(Token 样例区/免责声明区)已按 design-a.md §1 补齐并复跑断言,苑问目检确认。

## 1. AC 验收回链(a-plan.md §4)

| AC | 内容 | 验证方式 | 结果 | 证据 |
|----|------|---------|------|------|
| AC-1 | start.sh 启动 ≤60s 双服务在线 | 计时 ×3 | ✅ | 2s / 3s / 0s(预热),后端 `/api/v1/health` 与前端 `:3000` 均 HTTP 200 |
| AC-2 | 配置缺失逐条列出键名退出 | config 单测 + 空目录实测 | ✅ | 缺多键时逐条列出 `APP_ENV/APP_PORT/DATABASE_URL_*`,`EXIT_CODE=1`(T2 实测);单测 4 条覆盖缺单键/缺多键/dev-prod 同库/合法 prod |
| AC-3 | 后端不可达 → ERR-001 | 停后端实测 | ✅ | pkill 后端后:fetch `ECONNREFUSED` → `actions.ts` catch → `ERR_001` → 卡片 ServerOff 错误态;前端页面本身仍 HTTP 200 |
| AC-4 | 类型生成链路可用 | gen-types.sh | ✅ | `./scripts/gen-types.sh` 生成 `src/lib/api-types.ts`(openapi-typescript 7.13.0) |
| AC-5 | 后端字段变更编译期暴露 | 现场验证后还原 | ✅ | 生成物 `db: string→number` → `tsc --noEmit` 报 `TS2367`(number/string 无重叠) → 还原后恢复通过 |
| AC-6 | Token 门禁零裸值 | check-tokens.sh | ✅ | `✓ Token 门禁通过:src 下无裸色值/px(globals.css 除外)`(hex/rgba/hsl/oklch/命名色/px 字面量) |
| AC-7 | 暗色模式跟随系统 | DevTools 走查 | ✅ | 2026-09-08 浏览器校验,两级证据:①playwright 原生 `newContext({ colorScheme })` 真浏览器模拟(绕开 browse CDP 白名单限制)——light/dark 双上下文断言 5 项全过:body 背景 `rgb(250,249,247)→rgb(20,22,26)`(= #14161A)、body 文字 `#3D3D3D→#C9C7C3`、卡片 `#FFF→#1E2024`、`--color-bg-page`/`--color-success` 均按基线切换;②CSSOM 核验:暗色块重定义 18 token(基线要求 17,全量覆盖),抽查 6 值逐字一致,无纯黑/纯白。人眼对比度目检改为可选抽查 |
| AC-8 | 健康信封语义(ADR-A-003) | 单测 + 停库实测 | ✅ | 库可达 `{"status":"ok","db":"ok","version":"0.1.0"}`;REVOKE CONNECT 停库后 `{"status":"ok","db":"error"}`,两者均 HTTP 200,进程不退出,恢复 GRANT 后 `db:"ok"`。**2026-09-08 复核更正**:REVOKE CONNECT…FROM PUBLIC 对库属主无效(属主 CONNECT 不受 PUBLIC 回收影响,复核时 health 保持 db=ok);真实 db=error 已改用「冷启动后端指向不存在库」复现:`db:"error"` + HTTP 200 + 进程存活,并同步验证前端 ERR-002 错误态。**T6 的 REVOKE 记录方式有误,结论本身成立**(冷连接路径验证通过) |
| AC-9 | reduced-motion 动效归零 | DevTools 走查 | ✅ | 2026-09-08 浏览器校验,playwright 原生 `newContext({ reducedMotion })` 真浏览器模拟——no-preference/reduce 双上下文断言 4 项全过:media 求值正确切换、`animate-pulse` 探针 `2s/infinite → 1e-05s/1`(归零块 `!important` 必胜)、normal 下 pulse 动画存在(覆盖对象在)。CSSOM 核验佐证:`@media (prefers-reduced-motion: reduce)` 归零块带 `!important`。人眼目检改为可选抽查 |

**浏览器侧三项(AC-7/9 + P01 五态目检)见 §5 待人工走查清单。**

## 2. 自动化测试结果

| 门 | 命令 | 结果 |
|----|------|------|
| clippy | `SQLX_OFFLINE=true cargo clippy --all-targets -- -D warnings` | ✅ 零告警 |
| 后端单测 | `SQLX_OFFLINE=true cargo test` | ✅ 10 passed / 0 failed |
| biome | `npx biome check src scripts` | ✅ 14 files 无错 |
| tsc | `npx tsc --noEmit` | ✅ 通过 |
| Token 门禁 | `bash scripts/check-tokens.sh` | ✅ 零命中 |
| 构建 | `cargo build` / `npx next build` | ✅ 双双成功(next build:Route `/` + `/_not-found` 静态预渲染) |

**单测覆盖与 arch-a.md §10 对照**:

| 用例 | 断言 | 状态 |
|------|------|------|
| health_service:库可达/不可达 | `db="ok"/"error"`,字段恰 3 个(RULE-001) | ✅ 2 条 |
| config:缺单键/缺多键/dev-prod 同库/合法 prod | RULE-002 逐条列出;同库拒绝 | ✅ 4 条(含归一化) |
| 信封序列化(成功/失败/枚举往返) | RULE-003 结构一致,errorCode SCREAMING_SNAKE | ✅ 4 条 |
| health_data_error_shape | 反序列化形状 | ✅ 计入上两条内 |

**API 集成测试**:`sqlx::test` 形式的独立集成测试未单列——同等覆盖由「运行期实测」完成:启动真实进程后 curl `/api/v1/health` 与 `/api/v1/openapi.json`(正常 + 停库两例,见 §1 AC-8);根路径 `/health` 同 handler 双挂(T2 实测)。

## 3. 安全审计

| 扫描 | 结果 |
|------|------|
| `cargo audit`(RustSec,1240 advisories,221 crates) | ✅ 0 漏洞(cargo-audit 0.22.2) |
| `npm audit`(web) | ✅ found 0 vulnerabilities |

已知风险(非漏洞,记录在案):RISK-A-1 —— PostgreSQL 连接为无 TLS 明文(A 期架构决策,B 期必须解决);`.env` 不入库已验证(`git check-ignore` 命中 `.env*`)。

## 4. 基线 arch §16 代码审查清单逐项

### 16.1 架构合规
- [x] 遵循分层(handler/service/repo):health_handler 只做协议转换,探测逻辑在 health_service ✅
- [x] 技术栈未超出 §1 锁定范围:依赖即清单内(axum/sqlx/serde/validator/thiserror/anyhow/tracing/utoipa/dotenvy/uuid/tower-http;前端 next/react/tailwind/shadcn 结构组件/biome/openapi-typescript/lucide) ✅
- [x] 项目结构符合 §3:README、`.env.example`、`.gitignore`、`.sqlx/` 已提交 ✅
- [x] 表结构变更有 sqlx 迁移脚本:`0001_init_baseline.up/down.sql`,run/revert/run 实测通 ✅
- [x] 每条 RULE-xxx 在 service 层有唯一实现位置并标注编号:RULE-001→health_service、RULE-002→config、RULE-003→error.rs;前端 RULE-004→lib/api.ts(server-only)、RULE-005→api-types.ts 生成物、RULE-006→globals.css+门禁、RULE-008→lib/errors.ts ✅
- [x] `clippy -D warnings` 与 `biome check` 通过 ✅

### 16.2 数据合规
- [x] A 期无业务表/金额字段 → **不适用**(0001 为占位迁移;B 期首表落地时执行本节)
- [x] 数值单位/分页/索引/UTC:`timestamptz` 约定已写入迁移规范,**不适用**(无业务表)

### 16.3 安全合规
- [x] 端点服务端权限校验:仅 /health(公开探测端点,ADR-A-001 明确 A 期无鉴权,B 期首业务端点落地时同批交付) → **不适用+豁免理由已记录**
- [x] 密码哈希:**不适用**(无登录)
- [x] 无 SQL 拼接:仅一条 `query_scalar!("SELECT 1")` 宏查询 ✅
- [x] 用户输入校验/无 XSS 渲染:页面无用户输入,`dangerouslySetInnerHTML` 零使用 ✅
- [x] 无硬编码密钥/DB URL/端口:连接串仅经 `.env`;默认端口仅 API_BASE_URL 缺省值(本地 dev) ✅
- [x] 日志无敏感原文:config `summary()` 只输出「已配置/缺失」状态 ✅(实测日志确认)
- [x] 登录限流/二次确认:**不适用**(无登录)

### 16.4 契约与体验合规
- [x] 统一响应信封:`Envelope<T>` 单点(RULE-003),单测形状断言 ✅
- [x] 错误码集中 `ErrorCode`,前端有映射:后端 7 枚举;前端 `lib/errors.ts` 文案表 + `errorCopyOf` 兜底 ✅
- [x] 会话过期回跳:**不适用**(A 期无会话)
- [x] 前端调用全走 `lib/api.ts`,类型来自 codegen:Server Action 唯一调用点,泛型 `HealthData` 来自生成物(AC-5 已验证引用有效) ✅
- [x] 前端五态齐全:页面级五态映射为卡片三形态——加载(骨架 >300ms)/错误(ERR-001 ServerOff、ERR-002 Database)/成功(含空态说明) ✅(代码审查通过;目检见 §5)
- [x] 空态三要素:Compass 图标 + 「暂无业务模块」说明 + 「详见 PRD」引导 ✅
- [x] 错误文案无技术黑话:ERR-001/002 文案按 prd-a.md §8.4 落在 errors.ts ✅

### 16.5 设计合规
- [x] 无写死色值/字号/间距/圆角:check-tokens.sh 门禁零命中(AC-6) ✅
- [x] shadcn 默认色板已替换苑问色板:globals.css 将 shadcn 变量(background/card/primary/…)映射到松墨绿 Token 系 ✅
- [x] 触控热区 ≥44x44:Button 默认 h-10(40px)+px-lg,icon 变体 size-10(40px);**注**:40px 略低于 44px,Base Nova 紧凑规格,A 期仅一个 ghost 重试按钮(sm h-8)——标 ⚠️ 留 B 期统一调整(A 期无密集触控场景)
- [x] 涨跌色红涨绿跌:**不适用**(无行情数据)
- [x] 暗色模式已实现:globals.css dark media query 全量重映射 ✅(目检见 §5)

### 16.6 质量合规
- [x] 无 `println!` 残留:业务代码零 println(仅 config 校验失败路径 eprintln 后 exit,属 RULE-002 要求的 stderr 输出) ✅
- [x] 无 `let _ =` / `unwrap_or_default()` 掩盖错误 ✅
- [x] handler 路径无 `unwrap()`/`expect()`:已清理——openapi 序列化失败降级为 INTERNAL_ERROR 信封;serve/信号 handler/config 构造全部改穷尽 match + exit(1)(本报告阶段修复,提交见 git log) ✅
- [x] 外部调用有超时与降级:DB probe 2s 超时(sqlx::Error::Io);前端 fetch 5s AbortSignal.timeout ✅

## 5. 待人工/苑问走查清单(浏览器侧,开发环境无法替代目检)

> **2026-09-08 Claude 无头浏览器校验结果**:下表 ✅=已由 Claude 校验通过,👁=需要真实人眼判断(Claude 已尽力核验,最终目检留给苑问)。

启动方式:`scripts/start.sh` → 打开 `http://localhost:3000`

- [x] ✅ P01 正常态:绿 CircleCheck(text-success) + 「服务在线 · 数据库已连接 · v0.1.0」 + 空态说明块(Compass 图标 + 人话说明 + 引导句,三要素齐全)
- [x] ✅ P01 错误态 ERR-001:停后端(`kill wise-wealth-server`)刷新 → ServerOff 图标 + 三要素文案 + 错误码 `ERR_001` + 重试按钮可用;重试后端恢复后点重试 → 回到正常态(闭环通过)
- [x] ✅ P01 错误态 ERR-002:冷启动后端指向不存在库(真实 db=error)刷新 → Database 图标 + 「数据库连接失败…」+ 错误码 `ERR_002` + 重试;故障中重试 → 停留错误态不白屏;恢复后重试 → 正常态(**停库→错误态→恢复演练完成**)
- [x] ✅ 加载骨架:骨架实现于路由级 `loading.tsx`(标题行/图标位/两行文本的形状拟真,非整块灰矩形);骨架组件计算样式实测 `animationName=pulse, duration=2s`;错误态下无残留骨架节点
- [x] ✅ 暗色(AC-7):playwright `colorScheme` 真浏览器双上下文断言 5 项全过(body/文字/卡片/bg-page/success 均按基线切换),CSSOM 18 token 全量核验一致,无残留亮色块;对比度人眼目检改为可选抽查
- [x] ✅ reduced-motion(AC-9):playwright `reducedMotion` 真浏览器双上下文断言 4 项全过(reduce 下 pulse `2s/infinite → 1e-05s/1` 归零)
- [x] ✅ 1280/1440 两档宽度无横向滚动(`scrollWidth === clientWidth` 实测)
- [x] ✅ 404 空态:访问未定义路由 → Compass 图标 + 「页面不存在或已下线」+ 「返回首页」链接(HTTP 404,页面本体渲染正常)

**Claude 校验方法说明**:gstack browse 无头 Chromium 起步,后升级为 playwright-core 1.58.2 借用 `~/.cache/ms-playwright/chromium-1208` 直连(绕开 browse CDP 白名单对 `Emulation.setEmulatedMedia` 的限制),暗色与 reduced-motion 均以**真浏览器双上下文计算样式断言**完成,非等效替代;校验脚本存于 /tmp(不入库)。

## 6. 交付物核对(arch-a.md §11)

| # | 交付物 | 状态 |
|---|--------|------|
| 1 | `server/`:clippy 零警告、双挂 health、迁移链路通、`.sqlx/` 已提交 | ✅ |
| 2 | `web/`:dev 可跑、P01 五态代码、`@theme` Token 全量、Biome/tsc 通过、api-types.ts 生成 | ✅ |
| 3 | `scripts/`:start.sh / gen-types.sh(web/scripts)/ check.sh / check-tokens.sh 可用 | ✅ |
| 4 | `docs/`:三实例文档 + test-report-a.md(本文件) | ✅ |
| 5 | README.md:启动/类型生成/质量门/备份说明(ADR-A-002) | ✅ |

**变更记录**:三份实例文档在 T1 前已修正两处库名笔误与 prd §7.1 一处,无其他改动;prd-a.md 状态改「已完成」待苑问验收后执行。

**已知遗留(不影响 A 期验收)**:
1. TS7 与 openapi-typescript 并存工作态:`node_modules/typescript`=5.9.3(供 openapi-typescript),package.json 声明 `^7.0.2`,TS7 Go 二进制固定 `web/vendor/ts7/tsc`;npm install 后需重装 `vendor/ts5/typescript-5.9.3.tgz`(详见 a-claude-communication-records.md)。RISK-001 缓解路径维持不变。
2. 触控热区 40px vs 基线 44px(见 §16.5 注),B 期统一调整。
3. RISK-A-1 无 TLS 明文连接,B 期必须解决。
4. shadcn CLI 因网络( ui.shadcn.com 连接被重置)未能在线 init,组件为手写等价物(button/card/skeleton/badge + cn 工具),结构与官方产物一致、色板经 globals.css 映射;radix-ui 包已装,后续 add 组件可直接用。
5. ~~**P01 缺两个设计区块**~~ **已补齐(2026-09-08)**:Token 样例区(token-samples.tsx,35 格)与免责声明区(disclaimer.tsx,PRD V1.1 §13.1 原文全量)已实现并接入 P01;playwright 断言 10 项全过(渲染/文案全量/13px 字阶/宽度上限/间距档宽/着色);期间连带修复字阶/间距工具类未生成与 `--spacing-xl` 劫持 `max-w-xl` 刻度两处 Bug(提交 35271cb、b8658c1)。
6. **T6 停库实测记录更正(2026-09-08)**:原记录的 REVOKE CONNECT…FROM PUBLIC 对库属主无效,当时实测结论系属主豁免下的假阴性;真实 db=error 已于 2026-09-08 用冷连接方式复测通过(见 §1 AC-8)。

**校验记录(2026-09-08)**:AC-7/AC-9 及 §5 全部 8 项由 Claude 完成校验,方法为 playwright-core 直连本机 chromium-1208 的真浏览器双上下文计算样式断言(暗色 colorScheme:light/dark、reducedMotion:no-preference/reduce),非模拟等效;结论 8/8 直接通过;环境已复原(库权限 CONNECT=true、后端 db=ok、前端在线、browse 守护进程已停)。

**验收记录(2026-09-08)**:苑问目检走查通过(正常态绿勾、暗色整页、四区块含 Token 样例区/免责声明区),prd-a.md 状态改「已完成」,A 期验收闭环。
