import { cn } from "@/lib/utils";

/** shadcn Skeleton:加载态骨架(基线 §7.2——形状接近真实内容,禁止只用转圈)。 */
function Skeleton({ className, ...props }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="skeleton"
      className={cn("animate-pulse rounded-sm bg-bg-subtle", className)}
      {...props}
    />
  );
}

export { Skeleton };
