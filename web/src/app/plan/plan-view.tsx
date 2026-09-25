/**
 * P04 方案页五段(P04 五段结构见 prd-v1 §8.2;od-redesign 期按固化产物 p04-plan-github.html 重排)。
 *
 * 五段:分配总览 → 投资账户内部配置(L2) → 执行规则(L3) → 落地建议 → 风险提示。
 * 全部数字来自后端方案**快照**(`/api/v1/plans/active`),页面不重算、不猜测 ——
 * 页面与引擎不一致过一次,用户就再也不会信任这张表。
 *
 * 无客户端状态:生成/重试发生在 P03 步 6,本页只渲染既有方案,故是纯 server 组件
 * (基线 §7.4「能服务端渲染就不上客户端」)。
 */
import { AlertTriangle, Library, RefreshCw } from "lucide-react";
import Link from "next/link";

import { Disclaimer } from "@/components/disclaimer";
import { Badge } from "@/components/ui/badge";
import { buttonVariants } from "@/components/ui/button";
import { formatCurrency, formatMonthsTenths } from "@/lib/format-currency";
import { cn } from "@/lib/utils";

import { chartBgClass, CREDIBILITY_NOTICE, NOTICE_COPY, type PlanView as Plan } from "./state";

/**
 * 可信度提示条(RULE-035,F 期):disputed = 警示变体(attention 黄)、caution = 提示变体
 * (primary 蓝的 info 款)、verified / 字段缺失(模式已下架)→ 不渲染。
 * 文案唯一来源 = state.ts CREDIBILITY_NOTICE;评级读取时解析(ADR-F-002),
 * 描述的是模式的当前知识状态,不随历史快照冻结。
 */
function CredibilityNotice({ plan }: { plan: Plan }) {
  const c = plan.l1_credibility;
  if (c !== "disputed" && c !== "caution") return null;
  const copy = CREDIBILITY_NOTICE[c];
  const warn = copy.variant === "warn";
  return (
    <div
      role="status"
      className={cn(
        "flex items-start gap-base-md rounded-sm px-base-lg py-base-md",
        warn ? "bg-attention-subtle" : "bg-primary-soft",
      )}
    >
      <AlertTriangle
        aria-hidden="true"
        className={cn("mt-0.5 size-4 shrink-0", warn ? "text-warning" : "text-primary")}
      />
      <div className="flex flex-col gap-base-xs">
        <p className={cn("text-body-md font-semibold", warn ? "text-warning" : "text-primary")}>
          {copy.title}
        </p>
        <p className="text-caption text-ink-secondary">{copy.description}</p>
        {plan.l1_source ? (
          <p className="text-caption text-ink-mute">出处:{plan.l1_source}</p>
        ) : null}
      </div>
    </div>
  );
}

/**
 * 分区卡(产物 .plan-card):hairline 一像素边 + Canvas Subtle 头条 + 平面卡体。
 * 五段里「表格/规则/建议」类内容用它;风险提示是正文段,不走卡。
 */
function PlanCard({
  label,
  children,
  className,
}: {
  label: string;
  children: React.ReactNode;
  className?: string;
}) {
  return (
    <section
      className={cn(
        "flex flex-col overflow-hidden rounded-sm border border-hairline bg-canvas-card",
        className,
      )}
    >
      <div className="border-b border-hairline bg-canvas-soft px-base-lg py-base-sm text-caption font-semibold text-ink">
        {label}
      </div>
      <div className="flex flex-col gap-base-md p-base-lg">{children}</div>
    </section>
  );
}

/** 段标题(风险提示等正文段用)。 */
function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-base-md">
      <h2 className="text-heading-md text-ink">{title}</h2>
      {children}
    </section>
  );
}

/**
 * 提示条(产物 .notice):引擎给事实,文案取自 NOTICE_COPY(RULE-008 三要素)。
 * 严重 = Danger Subtle 底 + danger 标题;提醒 = Attention Subtle 底 + warning 标题
 * (两对底/字配色均为 GitHub DS 文档化组合,对比度过 4.5 线)。
 */
function NoticeBar({ notice }: { notice: keyof typeof NOTICE_COPY }) {
  const copy = NOTICE_COPY[notice];
  const severe = notice === "insufficient_income";
  return (
    <div
      role="status"
      className={cn(
        "flex items-start gap-base-md rounded-sm px-base-lg py-base-md",
        severe ? "bg-danger-subtle" : "bg-attention-subtle",
      )}
    >
      <AlertTriangle
        aria-hidden="true"
        className={cn("mt-0.5 size-4 shrink-0", severe ? "text-danger" : "text-warning")}
      />
      <div className="flex flex-col gap-base-xs">
        <p className={cn("text-body-md font-semibold", severe ? "text-danger" : "text-warning")}>
          {copy.title}
        </p>
        <p className="text-caption text-ink-secondary">{copy.description}</p>
      </div>
    </div>
  );
}

/**
 * 应急金状态(AC-25/26/31)。**不塞进表格单元格**:表格只放模式配置里的用途原文,
 * 状态类信息集中在这里 —— 否则同一事实(备用的用途)会有配置与状态两个来源。
 * 状态圆点(产物 .status-dot):绿 = 已达标,黄 = 未达标。
 */
function EmergencyPanel({ emergency }: { emergency: Plan["emergency"] }) {
  const coverage = formatMonthsTenths(emergency.coverage_tenths);
  return (
    <div className="flex flex-col gap-base-xs rounded-sm bg-canvas-soft px-base-lg py-base-md">
      <p className="flex items-center gap-base-sm text-body-md text-ink-secondary">
        <span
          aria-hidden="true"
          className={cn(
            "size-2 shrink-0 rounded-full",
            emergency.is_met ? "bg-success" : "bg-warning",
          )}
        />
        应急金:
        <span className={emergency.is_met ? "text-success" : "text-ink"}>
          {emergency.is_met ? "已达标" : "未达标"}
        </span>
        {" · "}目标 {emergency.months} 个月 = {formatCurrency(emergency.target_cents)}
        <span className="text-caption text-ink-mute-2">
          (必要月支出 {formatCurrency(emergency.necessary_monthly_cents)})
        </span>
      </p>
      <p className="text-caption text-ink-secondary">
        现有存款已覆盖约 <span className="font-mono tabular-nums">{coverage}</span> 个月
        {emergency.is_met
          ? ""
          : `,按每月 ${formatCurrency(emergency.monthly_toward_emergency_cents)} 的节奏,约 ${
              emergency.months_to_fill ?? 0
            } 个月补齐`}
      </p>
      {emergency.is_met ? (
        <p className="text-caption text-ink-secondary">
          {emergency.surplus_cents > 0
            ? `超出应急目标的部分不再单独预留:投资账户已有家底 ${formatCurrency(emergency.surplus_cents)}`
            : "超出应急目标的部分不再单独预留,份额自动进入投资账户"}
        </p>
      ) : null}
    </div>
  );
}

/** 一、分配总览(GitHub 密排表:Subtle 表头条 / hairline 行线 / 数值等宽右对齐 / 行高 44 触控)。 */
function OverviewSection({ plan }: { plan: Plan }) {
  return (
    <PlanCard label="一、分配总览">
      <div className="overflow-hidden rounded-sm border border-hairline">
        <table className="w-full border-collapse">
          <thead>
            <tr className="bg-canvas-soft text-caption text-ink-mute-2">
              <th scope="col" className="px-base-lg py-base-sm text-left">
                账户
              </th>
              <th scope="col" className="px-base-lg py-base-sm text-right">
                每月转入
              </th>
              <th scope="col" className="px-base-lg py-base-sm text-left">
                用途
              </th>
            </tr>
          </thead>
          <tbody>
            {plan.buckets.map((bucket, i) => (
              <tr key={bucket.bucket_id} className="border-t border-hairline">
                <th scope="row" className="h-11 px-base-lg text-left text-body-md font-normal">
                  <span className="flex items-center gap-base-sm text-ink">
                    <span
                      aria-hidden="true"
                      className={cn("size-2.5 shrink-0 rounded-full", chartBgClass(i))}
                    />
                    {bucket.name}
                  </span>
                </th>
                <td className="h-11 px-base-lg text-right text-body-tabular text-ink whitespace-nowrap">
                  {formatCurrency(bucket.amount_monthly_cents)}
                </td>
                <td className="h-11 px-base-lg text-caption text-ink-mute">{bucket.purpose}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <EmergencyPanel emergency={plan.emergency} />
    </PlanCard>
  );
}

/** 二、投资账户内部配置:横向比例条 + 文字图例并列(色盲可读,基线 §6.2 规则 2)。 */
function L2Section({ l2 }: { l2: Plan["l2"] }) {
  const legend = l2.classes
    .map((cls) => `${cls.name} ${Math.round(cls.basis_points / 100)}%`)
    .join("、");
  return (
    <PlanCard label="二、投资账户内部配置">
      <div className="flex flex-wrap items-center gap-base-sm">
        <Badge>{l2.name}</Badge>
        <span className="text-caption text-ink-mute">{l2.reason}</span>
      </div>

      {/* 宽度即占比(万分比 → %)。比例条是图形,宽度仍需内联 —— 工具类无法表达任意百分比。 */}
      <div
        role="img"
        aria-label={`投资账户配置比例:${legend}`}
        className="flex h-3 w-full overflow-hidden rounded-full bg-canvas-soft"
      >
        {l2.classes.map((cls, i) => (
          <span
            key={cls.name}
            className={cn("h-full", chartBgClass(i))}
            style={{ width: `${cls.basis_points / 100}%` }}
          />
        ))}
      </div>

      <ul className="flex flex-wrap gap-base-md">
        {l2.classes.map((cls, i) => (
          <li
            key={cls.name}
            className="flex items-center gap-base-xs text-caption text-ink-secondary tabular-nums"
          >
            <span
              aria-hidden="true"
              className={cn("size-2.5 shrink-0 rounded-full", chartBgClass(i))}
            />
            {cls.name} {Math.round(cls.basis_points / 100)}%
          </li>
        ))}
      </ul>

      {l2.note ? <p className="text-caption text-ink-mute">{l2.note}</p> : null}
      <p className="text-caption text-ink-mute">仅到大类资产,不涉及任何具体产品</p>
    </PlanCard>
  );
}

/** 三、执行规则(L3 自动给出,产品 PRD §4.3.2 三 + §7.3 现金再平衡为默认;编号等宽,产物样式)。 */
function ExecutionSection({ plan }: { plan: Plan }) {
  const rules = [
    "每月发薪日定投,金额按上表执行",
    "投资账户偏离目标比例 5% 时触发再平衡",
    "优先用新增资金补低配(现金再平衡,规避赎回费与择时)",
  ];
  return (
    <PlanCard label="三、执行规则">
      <div className="flex flex-col gap-base-sm text-body-md text-ink-secondary">
        {rules.map((rule, i) => (
          <p key={rule} className="flex items-baseline gap-base-sm">
            <span aria-hidden="true" className="font-mono text-ink-mute">
              {i + 1}.
            </span>
            {rule}
          </p>
        ))}
      </div>
      {plan.emergency.is_met ? null : (
        <p className="border-t border-dashed border-hairline pt-base-md text-caption text-ink-mute">
          应急金未达标期间,备用账户优先补应急(每月{" "}
          {formatCurrency(plan.emergency.monthly_toward_emergency_cents)},约{" "}
          {plan.emergency.months_to_fill ?? 0} 个月达标),达标后这部分份额转入投资账户。
        </p>
      )}
    </PlanCard>
  );
}

/**
 * 四、落地建议。文案按**桶派生**而非按模式写死:加一个模式不该回头改这个页面
 * (原型里的四条卡建议是四账户模式的示例,派生后对三桶模式同样成立)。
 */
function SuggestionSection({ plan }: { plan: Plan }) {
  const names = plan.buckets.map((b) => `${b.name}(${b.purpose})`).join(" / ");
  return (
    <PlanCard label="四、落地建议">
      <p className="text-body-md text-ink-secondary">
        建议开 2-3 张银行卡分别对应上表账户:{names}。资金到账后按上表金额分配。
      </p>
      <p className="text-caption text-ink-mute">仅为账户组织建议,不涉及任何划转操作</p>
    </PlanCard>
  );
}

export function PlanView({ plan }: { plan: Plan }) {
  return (
    <div className="flex flex-col gap-base-xxl">
      <header className="flex flex-col gap-base-xs">
        <h1 className="text-display-lg text-ink">你的方案</h1>
        <p className="text-caption text-ink-mute">
          {plan.l1_mode_name} · 第 <span className="font-mono tabular-nums">{plan.version}</span> 版
          · 生成于 <span className="font-mono tabular-nums">{plan.created_date}</span>
        </p>
      </header>

      <CredibilityNotice plan={plan} />

      {plan.notices.length > 0 ? (
        <div className="flex flex-col gap-base-sm">
          {plan.notices.map((notice) => (
            <NoticeBar key={notice} notice={notice} />
          ))}
        </div>
      ) : null}

      <OverviewSection plan={plan} />
      <L2Section l2={plan.l2} />
      <ExecutionSection plan={plan} />
      <SuggestionSection plan={plan} />

      <Section title="五、风险提示">
        {/* 声明全文的唯一来源是 lib/disclaimer-copy.ts;这里去掉组件的分隔线与宽度上限 ——
            它在方案页是「一段」而不是「页脚一块」 */}
        <Disclaimer className="mx-0 max-w-none border-t-0 pt-0" />
      </Section>

      <div className="flex flex-wrap gap-base-md">
        {/* 模式库入口(F 期 FEATURE-006):从这里能看见全部方法与它们的出处 */}
        <Link href="/modes" className={buttonVariants({ variant: "ghost" })}>
          <Library className="size-4" aria-hidden="true" />
          查看全部模式
        </Link>
        {/* 重新生成 = 回 P03 从步 1 走(restart=1;作答仍预填,只改变化的数字),再生成即版本 +1 */}
        <Link href="/questionnaire?restart=1" className={buttonVariants({ variant: "ghost" })}>
          <RefreshCw className="size-4" aria-hidden="true" />
          重新生成
        </Link>
        <Link href="/" className={buttonVariants({ variant: "ghost" })}>
          返回首页
        </Link>
      </div>
    </div>
  );
}
