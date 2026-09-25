/**
 * P06 模式库页视图(server 组件,F 期票 04)。视觉对齐固化产物
 * `docs/design/f-modes/p06-modes-github.html`:页头 + 卡片网格(桌面双列、窄屏单列)。
 *
 * 全部展示数据来自 GET /api/v1/modes 一次取回(ADR-F-004):名称/tagline/人群/
 * 可信度徽章/出处/桶概览(share_desc 已由服务端字符串化,前端零解读)。
 * 「推荐」徽章只由后端 is_recommended 决定(RULE-032/033:disputed 永不主推)。
 */
import { SiteFooterShell } from "@/components/site-footer-shell";

import { ModesBrowser } from "./modes-browser";
import type { ModesData } from "./state";

export function ModesView({
  data,
  initialSelected = [],
}: {
  data: ModesData;
  /** URL 回传的勾选态(AC-14:对比页返回后保持) */
  initialSelected?: string[];
}) {
  return (
    <>
      <main className="mx-auto w-full max-w-xl flex-1 px-base-lg py-base-xxl">
        <header className="flex flex-col gap-base-xs">
          <h1 className="text-display-lg text-ink">模式库</h1>
          <p className="text-caption text-ink-mute">每种分账方法都有出处与可信度评级</p>
        </header>

        <div className="mt-base-lg">
          <ModesBrowser items={data.items} initialSelected={initialSelected} />
        </div>
      </main>
      {/* P06 页脚沿 RULE-020/031:一行简述 + 完整声明展开(产物同款) */}
      <SiteFooterShell />
    </>
  );
}
