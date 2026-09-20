import { cookies } from "next/headers";
import Link from "next/link";

import { logoutAction } from "@/app/login/actions";
import { ThemeToggle } from "@/components/theme-toggle";
import { SESSION_COOKIE } from "@/lib/session";

/**
 * 站点顶部栏(design-v2 §30 / §96;od-redesign 产物 .topbar):sticky、hairline 底线;
 * 左侧品牌标(蓝色方标 + 名称)回 P01,右侧暗色切换按钮 + 登出按钮(登录后,产物 ghost 款)。
 *
 * 这里读 Cookie 只用于决定**是否显示登出** —— 令牌真伪由后端每个请求校验,
 * 顶部栏不是安全边界(基线 §16.3:权限校验在服务端,不依赖 UI 是否存在)。
 */
export async function TopBar() {
  const store = await cookies();
  const loggedIn = store.has(SESSION_COOKIE);

  return (
    <header className="sticky top-0 z-20 flex h-14 shrink-0 items-center justify-between gap-base-lg border-b border-hairline bg-canvas px-base-lg">
      {/* 触控热区 ≥44 像素(customInstructions):文字行高只有约 25 像素,靠 min-h 撑起来 */}
      <Link
        href="/"
        aria-label="智策理财,返回首页"
        className="inline-flex min-h-11 items-center gap-base-sm text-heading-sm font-semibold text-ink sm:min-h-10"
      >
        <span
          aria-hidden="true"
          className="grid size-5 shrink-0 place-items-center rounded-sm bg-primary text-on-primary"
        >
          <svg
            width="12"
            height="12"
            viewBox="0 0 16 16"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.8"
            strokeLinecap="round"
            strokeLinejoin="round"
            aria-hidden="true"
          >
            <path d="M2.5 11.5 6 7.5l2.5 2L13.5 4" />
          </svg>
        </span>
        智策理财
      </Link>
      <div className="flex items-center gap-base-xs">
        <ThemeToggle />
        {loggedIn ? (
          <form action={logoutAction}>
            <button
              type="submit"
              className="inline-flex min-h-11 items-center gap-base-sm rounded-sm border border-hairline bg-canvas-soft px-base-md text-body-md text-ink hover:bg-canvas-soft/60 sm:min-h-10"
            >
              登出
            </button>
          </form>
        ) : null}
      </div>
    </header>
  );
}
