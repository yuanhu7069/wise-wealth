import { cva, type VariantProps } from "class-variance-authority";
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/** shadcn Badge:主色浅底 + 主色文字(design-a.md §7),Pill 圆角。 */
const badgeVariants = cva(
  "inline-flex w-fit shrink-0 items-center justify-center gap-base-xs overflow-hidden rounded-full border border-transparent px-base-sm py-px text-label whitespace-nowrap",
  {
    variants: {
      variant: {
        default: "bg-primary-bg text-primary",
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
