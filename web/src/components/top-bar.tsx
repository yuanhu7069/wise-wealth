import { cookies } from "next/headers";
import Link from "next/link";

import { logoutAction } from "@/app/login/actions";
import { ThemeToggle } from "@/components/theme-toggle";
import { SESSION_COOKIE } from "@/lib/session";

/**
 * 站点顶部栏(design-v2 §30 / §96):左侧品牌回 P01,右侧暗色切换按钮 + 登出按钮(登录后)。
 * 高 h-14(移动 h-13)、bg-card + 底部 divider 细线;登出为文字按钮。
 *
 * 这里读 Cookie 只用于决定**是否显示登出** —— 令牌真伪由后端每个请求校验,
 * 顶部栏不是安全边界(基线 §16.3:权限校验在服务端,不依赖 UI 是否存在)。
 */
export async function TopBar() {
  const store = await cookies();
  const loggedIn = store.has(SESSION_COOKIE);

  return (
    <header className="flex h-13 shrink-0 items-center justify-between gap-base-lg border-b border-divider bg-card px-base-lg sm:h-14">
      {/* 触控热区取基线 §16 / design-v2 §43 要求的最小高度:18 像素文字自带高度只有约 25 像素,
          靠 min-h 撑起来 */}
      <Link
        href="/"
        className="inline-flex min-h-11 items-center text-section-title text-text-title sm:min-h-10"
      >
        智策理财
      </Link>
      <div className="flex items-center gap-base-xs">
        <ThemeToggle />
        {loggedIn ? (
          <form action={logoutAction}>
            <button
              type="submit"
              className="inline-flex min-h-11 items-center rounded-md px-base-sm text-body text-action hover:bg-accent sm:min-h-10"
            >
              登出
            </button>
          </form>
        ) : null}
      </div>
    </header>
  );
}
