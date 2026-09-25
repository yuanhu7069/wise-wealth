/**
 * P07 路由级加载态(G 期票 04)。对齐固化产物:页头 + 板块卡骨架,
 * 与 tracking/loading 同族写法(animate-pulse,Token 门禁友好)。
 */
export default function KnowledgeLoading() {
  return (
    <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl" aria-busy="true">
      <span className="sr-only" role="status">
        正在加载知识库…
      </span>
      <div aria-hidden="true" className="flex flex-col gap-base-lg">
        <div className="flex flex-col gap-base-xs">
          <div className="h-8 w-32 animate-pulse rounded-sm bg-canvas-soft" />
          <div className="h-4 w-72 animate-pulse rounded-sm bg-canvas-soft" />
        </div>
        {[0, 1].map((block) => (
          <div key={block} className="flex flex-col gap-base-sm">
            <div className="h-3 w-20 animate-pulse rounded-sm bg-canvas-soft" />
            <div className="h-20 animate-pulse rounded-sm border border-hairline bg-canvas-card" />
            <div className="h-20 animate-pulse rounded-sm border border-hairline bg-canvas-card" />
          </div>
        ))}
      </div>
    </main>
  );
}
