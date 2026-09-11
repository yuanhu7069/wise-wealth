import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * shadcn Button(基线 §7.2 按钮):主题经 globals.css CSS 变量映射到苑问色板。
 * ghost variant 供文字按钮(重试)使用——design-a.md §7。
 *
 * 两处 B 期修正(2026-09-11 走查,test-report-b.md §8-E / §8-B):
 * ① ghost 用 `text-action` 而非 `text-primary` —— primary 在暗色卡片底只有 4.34:1;
 * ② 尺寸在移动断点抬到 h-11,桌面回到 h-10(基线 §16 触控热区 / design-v2 §43)。
 */
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-body font-medium transition-colors disabled:pointer-events-none disabled:text-text-disabled [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary/90",
        secondary: "border border-border bg-card text-foreground hover:bg-muted",
        ghost: "text-action hover:bg-accent",
        destructive: "bg-destructive text-primary-foreground hover:bg-destructive/90",
      },
      size: {
        default: "h-11 px-base-lg sm:h-10",
        sm: "h-11 px-base-md sm:h-8",
        lg: "h-11 px-base-xl",
        icon: "size-11 sm:size-10",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

function Button({
  className,
  variant,
  size,
  ...props
}: ComponentProps<"button"> & VariantProps<typeof buttonVariants>) {
  return <button className={cn(buttonVariants({ variant, size, className }))} {...props} />;
}

export { Button, buttonVariants };
