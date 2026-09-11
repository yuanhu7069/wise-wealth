"use client";

/**
 * 登录表单(P02)。客户端组件只负责交互与错误展示,
 * 真正的登录与 Cookie 写入在 Server Action 里(基线 §5.1)。
 */
import { useActionState } from "react";

import { Button } from "@/components/ui/button";
import { loginAction } from "./actions";
import { initialLoginState } from "./state";

interface LoginFormProps {
  /** 登录成功后的回跳目标(已过站内白名单) */
  from: string;
}

export function LoginForm({ from }: LoginFormProps) {
  const [state, formAction, pending] = useActionState(loginAction, initialLoginState);

  return (
    <form action={formAction} className="flex flex-col gap-base-lg">
      <input type="hidden" name="from" value={from} />

      <div className="flex flex-col gap-base-xs">
        <label htmlFor="username" className="text-label font-medium text-text-title">
          用户名 <span className="text-danger">*</span>
        </label>
        <input
          id="username"
          name="username"
          autoComplete="username"
          className="h-11 rounded-sm border border-divider bg-bg-card px-base-md text-body text-text-body focus:border-primary focus:outline-none sm:h-10"
        />
      </div>

      <div className="flex flex-col gap-base-xs">
        <label htmlFor="password" className="text-label font-medium text-text-title">
          密码 <span className="text-danger">*</span>
        </label>
        <input
          id="password"
          name="password"
          type="password"
          autoComplete="current-password"
          aria-describedby={state.error ? "login-error" : undefined}
          className="h-11 rounded-sm border border-divider bg-bg-card px-base-md text-body text-text-body focus:border-primary focus:outline-none sm:h-10"
        />
        {state.error ? (
          // 错误文案与输入框程序化关联(design 基线 §10.7)
          <p id="login-error" role="alert" className="text-label text-danger">
            {state.error}
          </p>
        ) : null}
      </div>

      <Button type="submit" disabled={pending} className="w-full">
        {pending ? "登录中…" : "登录"}
      </Button>
    </form>
  );
}
