"use client";

import { Monitor, Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";

/**
 * 暗色三态切换按钮(design-v2 §3.3 / §97)。
 *
 * 三态循环:跟随系统 → 亮 → 暗 → 跟随系统。策略是 class —— `.light` / `.dark` 加在 `<html>` 上;
 * 「跟随系统」时**不加 class**,交给 globals.css 的 `@media (prefers-color-scheme: dark)` 接管
 * (这正是 design-v2 §3.3 要求的「与 A 期实现路径兼容」)。
 * 偏好存 localStorage:design-v2 §3.3 明确「非敏感,基线 §5.3 允许」(§5.3 禁的是存**凭证**)。
 */

const STORAGE_KEY = "ww-theme";
const ORDER = ["system", "light", "dark"] as const;

type ThemeChoice = (typeof ORDER)[number];

const LABEL: Record<ThemeChoice, string> = {
  system: "跟随系统",
  light: "亮色",
  dark: "暗色",
};

/** 把偏好落到 <html> 的 class 上。system = 两个 class 都去掉,让 media query 接管。 */
function applyTheme(choice: ThemeChoice): void {
  const root = document.documentElement;
  root.classList.toggle("light", choice === "light");
  root.classList.toggle("dark", choice === "dark");
}

function readStored(): ThemeChoice {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    return v === "light" || v === "dark" ? v : "system";
  } catch {
    // 隐私模式下读不到:按「跟随系统」处理
    return "system";
  }
}

export function ThemeToggle() {
  // 首帧固定 "system":若初值直接读 localStorage,服务端渲染的 aria-label 与客户端首帧会不一致。
  // 真实偏好在挂载后一次性同步(此时 <html> 的 class 已由 layout 的内联脚本设好,不会跳变)。
  const [choice, setChoice] = useState<ThemeChoice>("system");

  useEffect(() => {
    setChoice(readStored());
  }, []);

  const next = ORDER[(ORDER.indexOf(choice) + 1) % ORDER.length] ?? "system";

  function cycle(): void {
    applyTheme(next);
    try {
      if (next === "system") localStorage.removeItem(STORAGE_KEY);
      else localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // 写不进去:本次会话照样生效,只是记不住 —— 不必打断用户
    }
    setChoice(next);
  }

  // design-v2 §97 只点名了 Sun/Moon;「跟随系统」态取 Monitor(同属 lucide,语义即跟随系统)
  const Icon = choice === "dark" ? Moon : choice === "light" ? Sun : Monitor;
  const label = `切换主题(当前:${LABEL[choice]})`;

  return (
    <button
      type="button"
      onClick={cycle}
      aria-label={label}
      title={label}
      className="inline-flex size-11 items-center justify-center rounded-full text-action hover:bg-accent sm:size-10"
    >
      <Icon aria-hidden="true" size={24} />
    </button>
  );
}
