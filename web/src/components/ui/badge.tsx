import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * 参考件 `pill-tag-soft`(DESIGN.md「Pills, Tags, and Chips」):subdued 靛底 + press 靛字。
 *
 * 前景为什么走 `--accent-foreground` 而不是写死 primary-press:亮色取 press(对 subdued 底
 * 6.17:1)、暗色取深靛(9.1:1),由 globals.css 的 shadcn 映射层按模式切换,两态都过 4.5 线。
 * 字阶 `micro-cap`(参考件 10 像素上调一档,中文可读性,spec §2.2#5)。
 */
const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-base-xs overflow-hidden rounded-full border border-transparent px-base-sm py-px text-micro-cap whitespace-nowrap",
  {
    variants: {
      variant: {
        default: "bg-primary-bg-subdued-hover text-accent-foreground",
        secondary: "bg-canvas-soft text-ink-secondary",
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
