import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * shadcn Badge:主色浅底 + **accent 前景**(design-a.md §7),Pill 圆角。
 *
 * 前景为什么不是 `text-primary`:基线 §5.1.1 的 primary 在暗色浅绿底上只有 3.65:1,
 * 够不到正文 4.5:1 —— 2026-09-11 走查实测,见 test-report-b.md §8-E。
 * `--accent-foreground` 正是「浅绿底上的前景色」这项语义:亮色取 primary-deep、
 * 暗色取浅色前景,两个值都来自基线 §5.1.2。
 */
const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-base-xs overflow-hidden rounded-full border border-transparent px-base-sm py-px text-label whitespace-nowrap",
  {
    variants: {
      variant: {
        default: "bg-primary-bg text-accent-foreground",
        secondary: "bg-bg-subtle text-text-body",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

function Badge({
  className,
  variant,
  ...props
}: ComponentProps<"span"> & VariantProps<typeof badgeVariants>) {
  return (
    <span data-slot="badge" className={cn(badgeVariants({ variant }), className)} {...props} />
  );
}

export { Badge, badgeVariants };
