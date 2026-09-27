/**
 * P09 路由级骨架(对齐固化产物的加载态:切换器 + 三卡)。纯静态占位,无取数;
 * 动画由 globals.css 的 reduced-motion 全局规则兜底。
 */
export default function ReviewLoading() {
  return (
    <main className="mx-auto w-full max-w-[60rem] flex-1 px-base-lg py-base-xxl" aria-busy="true">
      <span className="sr-only" role="status">
        正在加载复盘数据…
      </span>
      <div aria-hidden="true" className="flex flex-col gap-base-lg">
        <div className="flex flex-col gap-base-xs">
          <div className="h-10 w-36 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-4 w-56 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="flex items-center gap-base-sm">
          <div className="h-11 w-24 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-11 flex-1 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-11 w-24 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="grid grid-cols-1 items-start gap-base-md md:grid-cols-[1fr_1.4fr_1fr]">
          {[0, 1, 2].map((i) => (
            <div
              key={i}
              className="flex flex-col gap-base-sm rounded-sm border border-hairline p-base-lg"
            >
              <div className="h-3 w-1/3 animate-pulse rounded-sm bg-canvas-soft" />
              <div className="h-9 w-2/5 animate-pulse rounded-sm bg-canvas-soft" />
              <div className="h-4 w-3/4 animate-pulse rounded-sm bg-canvas-soft" />
              <div className="h-4 w-2/3 animate-pulse rounded-sm bg-canvas-soft" />
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}
