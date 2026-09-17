/**
 * P02 登录页(server 壳)。职责只有两件:
 * ① 已登录时直接回跳(不让用户重复登录);② 把 after-whitelist 的 from 交给表单。
 */
import { redirect } from "next/navigation";

import { SiteFooterShell } from "@/components/site-footer-shell";
import { Card, CardContent } from "@/components/ui/card";
import { getSession, safeFrom } from "@/lib/session";

import { LoginForm } from "./login-form";

interface LoginPageProps {
  /** Next 以 Promise 形式提供 searchParams(Next 15+) */
  searchParams: Promise<{ from?: string; expired?: string }>;
}

export default async function LoginPage({ searchParams }: LoginPageProps) {
  const params = await searchParams;
  const raw = typeof params.from === "string" ? params.from : undefined;
  // RULE-001 / AC-4:from 只接受站内路径,外域与 // 开头一律丢弃
  const from = safeFrom(raw);

  const session = await getSession();
  if (session) redirect(from);

  return (
    <>
      <main className="flex-1 bg-canvas px-base-lg py-base-xxl">
        <Card className="mx-auto w-full max-w-md">
          <CardContent className="flex flex-col gap-base-lg pt-base-xl">
            <header className="flex flex-col gap-base-xs">
              <h1 className="text-heading-md text-ink">登录</h1>
              <p className="text-caption text-ink-mute">使用你的账号继续</p>
            </header>
            {/* ERR-006 辅助条:仅「有会话但已失效」时出现。首次访问也带着 from,
                但那时说「已过期」是错的 —— 判据来自 requireSession 的 expired 标记。 */}
            {params.expired ? (
              <p className="rounded-sm bg-canvas-soft px-base-md py-base-sm text-caption text-ink-mute-2">
                登录已过期,请重新登录
              </p>
            ) : null}
            <LoginForm from={from} />
          </CardContent>
        </Card>
      </main>
      <SiteFooterShell />
    </>
  );
}
