import type { Metadata, Viewport } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "智策理财",
  description: "双层理财决策工具:免费层告诉你怎么做",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

/**
 * 根布局只做骨架:html/body 与主题底色。
 *
 * **站点页脚不在这一层**:B 期页脚的承载位置按页不同(RULE-020)——P01/P02 显示站点页脚、
 * P03 只在操作栏内显示一行声明、P04 显示页脚但隐藏 legal 行。layout 拿不到当前路由,
 * 统一渲染必然把声明放到不该出现的位置(这正是 design-v2 v0.5/v0.6 记录的两次走查缺陷)。
 * 故页脚由各页自行渲染 `SiteFooterShell`,这里只保证「内容不足一屏时页脚落底」的列式骨架。
 */
export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="zh-CN" className="h-full antialiased">
      <body className="flex min-h-full flex-col bg-bg-page">
        <div className="flex flex-1 flex-col">{children}</div>
      </body>
    </html>
  );
}
