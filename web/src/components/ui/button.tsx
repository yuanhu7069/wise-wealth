import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * GitHub DS「Buttons」体系(od-redesign 固化产物 .btn):6 像素圆角、44 像素触控目标。
 * default = 实心绿主按钮(GitHub 绿,hover 深一档,白字,合同投影);
 * secondary = Outline Blue(白底蓝字,hover 填充蓝);
 * ghost = Default 灰(surface 底 + hairline 边,hover 按合同加深一档);
 * destructive = 危险 outline(白底红字,hover 填充深红白字)。
 *
 * 移动端 h-11 保证触控目标不小于 44 像素(customInstructions,从严于 GitHub 实际 32 像素)。
 */
const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-sm border border-transparent text-button-md transition-colors disabled:pointer-events-none disabled:text-ink-mute-2 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "border-btn-border bg-btn-primary text-on-primary shadow-btn-primary hover:bg-btn-primary-hover",
        secondary:
          "border-hairline bg-canvas text-primary hover:bg-btn-primary hover:text-on-primary",
        ghost: "border-hairline bg-canvas-soft text-ink hover:bg-btn-ghost-hover",
        destructive:
          "border-hairline bg-canvas text-danger hover:border-btn-danger-hover hover:bg-btn-danger-hover hover:text-on-primary",
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
