/**
 * Token 样例区(design-a.md §1 第 3 条):色板/字阶/间距/圆角/阴影五组静态样例。
 * 每格下方以 --text-label 标注 Token 名,B 期开发对照用(design 基线红线:唯一值来源 = 基线 §5)。
 * 样例本身只引用工具类,禁止裸值(Token 门禁 AC-6)。
 */
import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

function SampleCell({
  name,
  className,
  swatchClassName,
  ...props
}: ComponentProps<"div"> & { name: string; swatchClassName?: string }) {
  return (
    <div className="flex flex-col items-start gap-base-xs">
      <div
        aria-hidden="true"
        className={cn("size-base-xl rounded-sm border border-divider", swatchClassName)}
      />
      <code className="text-label text-text-aux">{name}</code>
      {props.children}
    </div>
  );
}

function Group({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-base-md">
      <h2 className="text-section-title text-text-title">{title}</h2>
      <div className="flex flex-wrap gap-base-md">{children}</div>
    </section>
  );
}

export function TokenSamples({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="token-samples"
      className={cn("flex flex-col gap-base-xxl", className)}
      {...props}
    >
      <Group title="色板(基线 §5.1.1)">
        <SampleCell name="--color-primary" swatchClassName="bg-primary" />
        <SampleCell name="--color-primary-deep" swatchClassName="bg-primary-deep" />
        <SampleCell name="--color-primary-light" swatchClassName="bg-primary-light" />
        <SampleCell name="--color-primary-bg" swatchClassName="bg-primary-bg" />
        <SampleCell name="--color-success" swatchClassName="bg-success" />
        <SampleCell name="--color-warning" swatchClassName="bg-warning" />
        <SampleCell name="--color-danger" swatchClassName="bg-danger" />
        <SampleCell name="--color-bg-page" swatchClassName="bg-bg-page" />
        <SampleCell name="--color-bg-card" swatchClassName="bg-bg-card" />
        <SampleCell name="--color-bg-subtle" swatchClassName="bg-bg-subtle" />
        <SampleCell name="--color-divider" swatchClassName="bg-divider" />
        <SampleCell name="--color-text-title" swatchClassName="bg-text-title" />
        <SampleCell name="--color-text-body" swatchClassName="bg-text-body" />
        <SampleCell name="--color-text-aux" swatchClassName="bg-text-aux" />
        <SampleCell name="--color-up" swatchClassName="bg-up" />
        <SampleCell name="--color-down" swatchClassName="bg-down" />
      </Group>

      <Group title="字阶(基线 §5.2,六档)">
        <div className="flex flex-col gap-base-xs">
          <span className="text-page-title text-text-title">智策理财</span>
          <code className="text-label text-text-aux">--text-page-title / 600</code>
        </div>
        <div className="flex flex-col gap-base-xs">
          <span className="text-section-title text-text-title">区块标题</span>
          <code className="text-label text-text-aux">--text-section-title / 600</code>
        </div>
        <div className="flex flex-col gap-base-xs">
          <span className="text-body text-text-body">正文内容示例</span>
          <code className="text-label text-text-aux">--text-body / 400</code>
        </div>
        <div className="flex flex-col gap-base-xs">
          <span className="text-aux text-text-aux">辅助说明示例</span>
          <code className="text-label text-text-aux">--text-aux / 400</code>
        </div>
        <div className="flex flex-col gap-base-xs">
          <span className="text-label text-text-body">标签示例</span>
          <code className="text-label text-text-aux">--text-label / 500</code>
        </div>
        <div className="flex flex-col gap-base-xs">
          <span className="text-data tabular-nums text-text-title">1,234.56</span>
          <code className="text-label text-text-aux">--text-data / 500 · tabular-nums</code>
        </div>
      </Group>

      <Group title="间距(基线 §5.3,4 倍数基准六档)">
        <SampleCell name="--space-xs" swatchClassName="w-base-xs" />
        <SampleCell name="--space-sm" swatchClassName="w-base-sm" />
        <SampleCell name="--space-md" swatchClassName="w-base-md" />
        <SampleCell name="--space-lg" swatchClassName="w-base-lg" />
        <SampleCell name="--space-xl" swatchClassName="w-base-xl" />
        <SampleCell name="--space-xxl" swatchClassName="w-base-xxl" />
      </Group>

      <Group title="圆角(基线 §5.4)">
        <SampleCell name="--radius-sm" swatchClassName="rounded-sm" />
        <SampleCell name="--radius-md" swatchClassName="rounded-md" />
        <SampleCell name="--radius-lg" swatchClassName="rounded-lg" />
        <SampleCell name="--radius-full" swatchClassName="rounded-full" />
      </Group>

      <Group title="阴影(基线 §5.5,三级)">
        <SampleCell name="--shadow-card" swatchClassName="shadow-card" />
        <SampleCell name="--shadow-float" swatchClassName="shadow-float" />
        <SampleCell name="--shadow-modal" swatchClassName="shadow-modal" />
      </Group>
    </div>
  );
}
