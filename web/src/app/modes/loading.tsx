/**
 * P06 路由级加载态(F 期票 04)。对齐固化产物的加载骨架段:
 * 页头 + 双卡形骨架(名称/徽章/tagline/人群/展开按钮),纯静态占位无取数;
 * 动画时长由 globals.css 的 reduced-motion 全局规则兜底(与 tracking 同族写法)。
 */
export default function ModesLoading() {
  return (
    <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl" aria-busy="true">
      <span className="sr-only" role="status">
        正在加载模式库…
      </span>
      <div aria-hidden="true" className="flex flex-col gap-base-lg">
        <div className="flex flex-col gap-base-xs">
          <div className="h-8 w-40 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-4 w-64 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="grid gap-base-md sm:grid-cols-2">
          {[0, 1].map((i) => (
            <div
              key={i}
              className="flex flex-col rounded-sm border border-hairline bg-canvas-card p-base-lg"
            >
              <div className="flex items-center justify-between gap-base-sm">
                <div
                  className={cn(
                    "h-5 animate-pulse rounded-sm bg-canvas-soft",
                    i === 0 ? "w-1/2" : "w-2/5",
                  )}
                />
                <div className="h-5 w-16 animate-pulse rounded-full bg-canvas-soft" />
              </div>
              <div
                className={cn(
                  "mt-base-sm h-4 animate-pulse rounded-sm bg-canvas-soft",
                  i === 0 ? "w-3/5" : "w-4/5",
                )}
              />
              <div className="mt-base-md h-5 w-[46%] animate-pulse rounded-full bg-canvas-soft" />
              <div className="mt-base-md h-4 w-[30%] animate-pulse rounded-sm bg-canvas-soft" />
            </div>
          ))}
        </div>
      </div>
    </main>
  );
}

function cn(...classes: Array<string | false | undefined>): string {
  return classes.filter(Boolean).join(" ");
}
