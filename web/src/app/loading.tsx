import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

/** P01 路由级 loading(基线 §7.2 骨架屏):形状接近摘要卡,禁止一整块灰矩形。 */
export default function Loading() {
  return (
    <main
      className="mx-auto flex w-full max-w-xl flex-1 flex-col gap-base-lg px-base-lg py-base-xxl"
      aria-busy="true"
    >
      <header className="flex flex-col gap-base-xs">
        <Skeleton className="h-8 w-56" />
        <Skeleton className="h-4 w-48" />
      </header>
      <Card aria-label="正在读取方案">
        <CardContent className="flex flex-col gap-base-md p-base-xl">
          <Skeleton className="h-6 w-44" />
          <Skeleton className="h-8 w-40" />
          <Skeleton className="h-12 w-full" />
          <div className="flex flex-wrap gap-base-md">
            <Skeleton className="h-11 w-32" />
            <Skeleton className="h-11 w-28" />
          </div>
        </CardContent>
      </Card>
    </main>
  );
}
