import type { Metadata, Viewport } from "next";
import "./globals.css";

import { SiteFooter } from "@/components/site-footer";
import { apiGet, type HealthData } from "@/lib/api";

export const metadata: Metadata = {
  title: "智策理财",
  description: "双层理财决策工具:免费层告诉你怎么做",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

/**
 * 页脚的健康状态在 layout 里取:它是运维指示器,与具体业务页无关。
 * 取不到(后端未启动)时降级为黄点 —— 不让一个探测失败把整页打崩(基线 §10.2)。
 */
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

export default async function RootLayout({ children }: LayoutProps<"/">) {
  const status = await footerStatus();

  return (
    <html lang="zh-CN" className="h-full antialiased">
      <body className="flex min-h-full flex-col bg-bg-page">
        <div className="flex-1">{children}</div>
        <SiteFooter healthy={status.healthy} version={status.version} />
      </body>
    </html>
  );
}
