/**
 * P08 路由级加载态(G 期票 06)。表格骨架(表头 + 行),同族写法。
 */
export default function CompareLoading() {
  return (
    <main className="mx-auto w-full max-w-6xl flex-1 px-base-lg py-base-xxl" aria-busy="true">
      <span className="sr-only" role="status">
        正在计算对比…
      </span>
      <div aria-hidden="true" className="flex flex-col gap-base-lg">
        <div className="flex flex-col gap-base-xs">
          <div className="h-8 w-32 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-4 w-56 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        <div className="overflow-x-auto rounded-sm border border-hairline">
          <div className="flex min-w-[36rem] flex-col">
            <div className="flex gap-px bg-hairline">
              {[0, 1, 2, 3].map((i) => (
                <div key={i} className="h-11 flex-1 animate-pulse bg-canvas-soft" />
              ))}
            </div>
            {[0, 1, 2, 3, 4].map((r) => (
              <div key={r} className="flex gap-px bg-hairline">
                {[0, 1, 2, 3].map((c) => (
                  <div
                    key={c}
                    className={`h-14 flex-1 animate-pulse ${c === 0 ? "bg-canvas-soft" : "bg-canvas"}`}
                  />
                ))}
              </div>
            ))}
          </div>
        </div>
      </div>
    </main>
  );
}
