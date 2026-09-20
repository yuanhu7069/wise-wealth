import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * GitHub DS「Labels」(od-redesign 产物 .badge):Accent Subtle 底 + Primary 字,pill 圆角,600 字重。
 *
 * 前景为什么走 `--accent-foreground` 而不是写死 primary:亮色取 primary(对 Accent Subtle 底
 * 4.58:1)、暗色取派生亮蓝,由 globals.css 的 shadcn 映射层按模式切换,两态都过 4.5 线。
 * 字阶 `micro-cap`(12 像素,GitHub 12 像素下限,RULE-020 声明条 11 像素下限之上)。
 */
const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-base-xs overflow-hidden rounded-full border border-transparent px-base-sm py-px text-micro-cap font-semibold whitespace-nowrap",
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
