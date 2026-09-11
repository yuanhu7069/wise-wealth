import type { Metadata, Viewport } from "next";

import { TopBar } from "@/components/top-bar";

import "./globals.css";

export const metadata: Metadata = {
  title: "智策理财",
  description: "双层理财决策工具:免费层告诉你怎么做",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
};

/** 主题偏好的 localStorage 键,与 components/theme-toggle.tsx 共用同一字符串。 */
const THEME_STORAGE_KEY = "ww-theme";

/**
 * 首帧防闪烁(design-v2 §3.3):在文档解析阶段就把已保存的主题落到 `<html>` 的 class 上,
 * 否则会先按亮色画一帧再切暗色。内容全是字面量、不含用户输入,故无注入面。
 */
const THEME_INIT = `try{var t=localStorage.getItem("${THEME_STORAGE_KEY}");if(t==="light"||t==="dark"){document.documentElement.classList.add(t)}}catch(e){}`;

/**
 * 根布局:html/body 骨架 + 站点头部。
 *
 * **站点页脚不在这一层**:B 期页脚的承载位置按页不同(RULE-020)——P01/P02 显示站点页脚、
 * P03 只在操作栏内显示一行声明、P04 显示页脚但隐藏 legal 行。layout 拿不到当前路由,
 * 统一渲染必然把声明放到不该出现的位置(这正是 design-v2 v0.5/v0.6 记录的两次走查缺陷)。
 * 故页脚由各页自行渲染 `SiteFooterShell`,这里只保证「内容不足一屏时页脚落底」的列式骨架。
 *
 * 头部(`TopBar`)则是全站同一份,放这一层;它读会话 Cookie 决定要不要给「登出」,
 * 因而全站按动态渲染处理(cookies() 的结果不可静态化)。
 * `suppressHydrationWarning` 是为 THEME_INIT 加的:脚本会在 hydration 前改 `<html>` 的 class。
 */
export default function RootLayout({ children }: LayoutProps<"/">) {
  return (
    <html lang="zh-CN" className="h-full antialiased" suppressHydrationWarning>
      <head>
        {/* biome-ignore lint/security/noDangerouslySetInnerHtml: THEME_INIT 是构建期字面量常量,不含用户输入 */}
        <script dangerouslySetInnerHTML={{ __html: THEME_INIT }} />
      </head>
      <body className="flex min-h-full flex-col bg-bg-page">
        <TopBar />
        <div className="flex flex-1 flex-col">{children}</div>
      </body>
    </html>
  );
}
