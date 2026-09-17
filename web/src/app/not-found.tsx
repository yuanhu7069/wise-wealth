import { Compass } from "lucide-react";

/** 404 空态(基线 §7.3):图标 + 人话说明 + 明确引导。 */
export default function NotFound() {
  return (
    <main className="mx-auto flex w-full max-w-xl flex-1 flex-col items-center justify-center gap-base-md px-base-lg py-base-xxl text-center">
      <Compass className="size-10 text-ink-mute" aria-hidden="true" />
      <p className="text-body-md text-ink-secondary">页面不存在或已下线。</p>
      <p className="text-caption text-ink-mute">请检查网址,或回到首页查看服务状态。</p>
      <a href="/" className="text-body-md font-medium text-primary hover:underline">
        返回首页
      </a>
    </main>
  );
}
