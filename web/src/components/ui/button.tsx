import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * 参考件 `button-primary-pill` 体系(DESIGN.md「Buttons」):全部 pill 圆角,
 * 实心靛蓝每区块至多一个;hover deep / active press。
 * ghost variant 供文字按钮(重试/重新生成)使用;destructive 底用 danger-strong(白字达标)。
 *
 * 移动断点抬到 h-11 保证触控目标不小于 44 像素(DESIGN.md「Touch Targets」,从严于参考)。
 */
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-full text-button-md transition-colors disabled:pointer-events-none disabled:text-ink-mute-2 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default: "bg-primary text-on-primary hover:bg-primary-deep active:bg-primary-press",
        secondary: "border border-primary bg-canvas text-primary hover:bg-accent",
        ghost: "text-action hover:bg-accent",
        destructive: "bg-danger-strong text-on-primary hover:bg-danger/90",
      },
      size: {
        default: "h-11 px-base-lg sm:h-10",
        sm: "h-11 px-base-lg text-button-sm sm:h-9",
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
