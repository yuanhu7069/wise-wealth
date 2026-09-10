/**
 * 免责声明区(design-a.md §1 第 4 条):智策产品 PRD §13.1 风险提示模板**原文全量**,
 * --text-aux 字阶,720 阅读宽度上限居中(design 基线 §6.4 第 1 条,规格值非样式取值)。
 *
 * 文案唯一来源已收敛到 `lib/disclaimer-copy.ts`(RULE-020):P01/P02 站点页脚、P03 声明条、
 * P04 风险提示段与本书面组件共用同一段字符串,禁止任一处各自抄写(改动只能来自 PRD §13.1 改版)。
 *
 * 头部品牌旁与免责声明区不放图标(design-a.md §5 克制原则)。
 * 注:本文件数值仅存在于注释(Token 门禁 RULE-006 扫源码取值,注释豁免属预期)。
 */
import type { ComponentProps } from "react";

import { DISCLAIMER_FULL } from "@/lib/disclaimer-copy";
import { cn } from "@/lib/utils";

export function Disclaimer({ className, ...props }: ComponentProps<"footer">) {
  return (
    <footer
      className={cn("mx-auto w-full max-w-2xl border-t border-divider pt-base-lg", className)}
      {...props}
    >
      <p className="text-aux text-text-aux">{DISCLAIMER_FULL}</p>
    </footer>
  );
}
