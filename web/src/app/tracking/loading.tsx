/**
 * P05 路由级骨架(对齐固化产物的加载态:录入卡 + 进度 + 列表 + 导出四段)。
 * 纯静态占位,无取数;动画时长由 globals.css 的 reduced-motion 全局规则兜底。
 */
export default function TrackingLoading() {
  return (
    <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl" aria-busy="true">
      <span className="sr-only" role="status">
        正在加载追踪数据…
      </span>
      <div aria-hidden="true" className="flex flex-col gap-base-xl">
        <div className="flex flex-col gap-base-xs">
          <div className="h-8 w-32 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-4 w-24 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="flex flex-col gap-base-md rounded-sm border border-hairline p-base-lg">
          <div className="grid grid-cols-2 gap-base-sm">
            {[0, 1, 2, 3].map((i) => (
              <div key={i} className="flex flex-col gap-base-xs">
                <div className="h-3 w-1/3 animate-pulse rounded-sm bg-canvas-soft" />
                <div className="h-11 animate-pulse rounded-sm bg-canvas-soft" />
              </div>
            ))}
          </div>
          <div className="mt-base-sm h-11 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="flex flex-col gap-base-sm rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md">
          <div className="h-4 w-3/5 animate-pulse rounded-sm bg-canvas" />
          <div className="h-4 w-2/5 animate-pulse rounded-sm bg-canvas" />
        </div>
        <div className="flex flex-col gap-base-sm rounded-sm border border-hairline p-base-lg">
          {[0, 1].map((i) => (
            <div
              key={i}
              className="flex flex-col gap-base-xs border-b border-hairline pb-base-md last:border-b-0"
            >
              <div className="h-4 w-2/5 animate-pulse rounded-sm bg-canvas-soft" />
              <div className="grid grid-cols-2 gap-base-sm">
                {[0, 1, 2, 3].map((j) => (
                  <div key={j} className="h-5 animate-pulse rounded-sm bg-canvas-soft" />
                ))}
              </div>
            </div>
          ))}
        </div>
        <div className="flex gap-base-sm">
          <div className="h-11 flex-1 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-11 flex-1 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
      </div>
    </main>
  );
}
