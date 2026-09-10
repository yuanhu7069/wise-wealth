/**
 * P01 唯一页面(基线 §7.1):A 期验证页——Token 体系与状态实现模式打样。
 * Server 组件只做布局与标题,状态逻辑收敛在 Client 卡片(经 Server Action 取数)。
 */
import { fetchHealthAction } from "@/app/actions";
import { SiteFooterShell } from "@/components/site-footer-shell";
import { HealthStatusCard } from "@/components/health-status-card";
import { requireSession } from "@/lib/session";

export default async function Page() {
  // RULE-001:P01 属受保护页面,未登录跳登录页并在登录后回跳
  await requireSession("/");

  return (
    <>
      <main className="mx-auto flex w-full max-w-xl flex-1 flex-col gap-base-lg px-base-lg py-base-xxl">
        <header className="flex flex-col gap-base-xs">
          <h1 className="text-page-title text-text-title">智策理财</h1>
          <p className="text-aux text-text-aux">A 期工程骨架 · 服务状态</p>
        </header>
        <HealthStatusCard fetchHealth={fetchHealthAction} />
      </main>
      <SiteFooterShell />
    </>
  );
}
