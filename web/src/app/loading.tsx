import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";

/** P01 路由级 loading(基线 §7.2 骨架屏):形状接近真实卡片,禁止一整块灰矩形。 */
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
      <Card aria-label="正在加载服务状态">
        <CardContent className="flex flex-col gap-base-md p-base-xl">
          <div className="flex items-center gap-base-md">
            <Skeleton className="size-6 rounded-sm" />
            <Skeleton className="h-6 w-64 max-w-full" />
          </div>
          <Skeleton className="h-5 w-96 max-w-full" />
          <Skeleton className="h-5 w-72 max-w-full" />
        </CardContent>
      </Card>
    </main>
  );
}
