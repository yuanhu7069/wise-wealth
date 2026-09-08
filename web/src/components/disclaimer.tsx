/**
 * 免责声明区(design-a.md §1 第 4 条):智策产品 PRD §13.1 风险提示模板**原文全量**,
 * --text-aux 字阶,720 阅读宽度上限居中(design 基线 §6.4 第 1 条,规格值非样式取值)。
 * 唯一文案来源 = docs/智策理财_PRD_V1.1.md §13.1;禁止改写、摘编(design-a.md §8 验收表)。
 * 头部品牌旁与免责声明区不放图标(design-a.md §5 克制原则)。
 * 注:本文件数值仅存在于注释(Token 门禁 RULE-006 扫源码取值,注释豁免属预期)。
 */
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

export function Disclaimer({ className, ...props }: ComponentProps<"footer">) {
  return (
    <footer
      className={cn("mx-auto w-full max-w-2xl border-t border-divider pt-base-lg", className)}
      {...props}
    >
      <p className="text-aux text-text-aux">
        <strong className="font-medium text-text-body">免责声明</strong>
        :本产品提供的所有理财方案和建议均为基于公开理财理论的
        <strong className="font-medium text-text-body">参考性建议</strong>
        ,不构成任何形式的投资建议、收益承诺或金融产品营销。所有涉及未来收益的内容均为
        <strong className="font-medium text-text-body">基于假设的模拟推演</strong>
        ,历史数据不代表未来表现。市场有风险,投资需谨慎。用户应根据自身实际情况独立做出财务决策,并自行承担相应风险。本产品不涉及任何形式的资金划转、托管或代持。
      </p>
    </footer>
  );
}
