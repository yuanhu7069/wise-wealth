/**
 * 站点页脚(RULE-020):健康状态 + **一行**免责声明,完整原文折叠在「完整声明」后。
 *
 * 两处取舍(已记入 prd-v1 RULE-020):
 * - 默认只显示一行:页脚要轻,原文的常驻可见性让位于此,一步可达即可;
 * - 用原生 `<details>` 而非客户端状态:折叠是浏览器原生能力,不必为此引入
 *   "use client"(design 基线 §7.4 组件复用规则、§8.2 少动效)。
 *
 * 字号取 `text-caption` 档(DESIGN.md「Typography」caption 档);页脚说明类文字不上更小档。
 */
import { ChevronDown } from "lucide-react";

import { DISCLAIMER_FULL, LEGAL_LINE } from "@/lib/disclaimer-copy";
import { cn } from "@/lib/utils";

interface SiteFooterProps {
  /** 后端版本号;拿不到时不显示这一项 */
  version?: string;
  /** 健康状态:ok 显示绿点,否则黄点 */
  healthy?: boolean;
  /** P04 的风险提示是方案五段之一,页脚不再重复同一段文字 */
  hideLegal?: boolean;
  className?: string;
}

export function SiteFooter({
  version,
  healthy = true,
  hideLegal = false,
  className,
}: SiteFooterProps) {
  return (
    <footer
      className={cn(
        "border-t border-hairline bg-canvas px-base-lg pt-base-lg pb-base-xxl",
        className,
      )}
    >
      <div className="mx-auto flex max-w-7xl flex-col gap-base-xs">
        <p className="flex items-center gap-base-xs text-caption text-ink-mute">
          <span
            aria-hidden="true"
            className={cn("size-2 rounded-full", healthy ? "bg-success" : "bg-warning")}
          />
          服务在线{version ? ` · v${version}` : ""}
        </p>

        {hideLegal ? null : (
          <details className="group text-caption text-ink-mute">
            <summary className="flex cursor-pointer list-none items-center gap-base-xs marker:content-none">
              <span>{LEGAL_LINE}</span>
              <span className="flex shrink-0 items-center gap-base-xs underline underline-offset-2">
                <span className="group-open:hidden">完整声明</span>
                <span className="hidden group-open:inline">收起</span>
                <ChevronDown
                  className="size-3 transition-transform group-open:rotate-180"
                  aria-hidden="true"
                />
              </span>
            </summary>
            <p className="pt-base-sm leading-relaxed">{DISCLAIMER_FULL}</p>
          </details>
        )}
      </div>
    </footer>
  );
}
