/**
 * 站点页脚的取数壳(RULE-020)。
 *
 * 页脚在 B 期的承载位置**按页不同**:P01/P02 = 站点页脚;P03 = 操作栏内声明条(无页脚);
 * P04 = 页脚但隐藏 legal 行(风险提示已是五段之一,同屏不重复同一段文字)。
 * root layout 无法得知当前路由,故页脚改由各页自行渲染,取数与降级逻辑收敛在这里。
 *
 * 健康状态取不到(后端未启动)时降级为黄点 —— 不让一个探测失败把整页打崩(基线 §10.2)。
 */
import { SiteFooter } from "@/components/site-footer";
import { apiGet, type HealthData } from "@/lib/api";

async function footerStatus(): Promise<{ healthy: boolean; version?: string }> {
  try {
    const { envelope } = await apiGet<HealthData>("/api/v1/health");
    return {
      healthy: envelope.success && envelope.data?.db === "ok",
      version: envelope.data?.version,
    };
  } catch {
    return { healthy: false };
  }
}

export async function SiteFooterShell({
  /** P04 传 true:legal 行由方案五段之「五、风险提示」承载 */
  hideLegal = false,
}: {
  hideLegal?: boolean;
}) {
  const status = await footerStatus();
  return <SiteFooter healthy={status.healthy} version={status.version} hideLegal={hideLegal} />;
}
