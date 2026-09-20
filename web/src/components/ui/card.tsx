import type { ComponentProps } from "react";

import { cn } from "@/lib/utils";

/**
 * GitHub DS「Cards / Boxes」(od-redesign 产物 .card):canvas-card 底、hairline 一像素边、
 * 6 像素圆角、无投影(GitHub 卡面平,层次靠 hairline)。禁止卡片套卡片。
 */
function Card({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="card"
      className={cn(
        "rounded-sm border border-hairline bg-canvas-card text-ink-secondary",
        className,
      )}
      {...props}
    />
  );
}

function CardHeader({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="card-header"
      className={cn("flex flex-col gap-base-sm p-base-xl", className)}
      {...props}
    />
  );
}

function CardTitle({ className, ...props }: ComponentProps<"h3">) {
  return (
    <h3 data-slot="card-title" className={cn("text-heading-sm text-ink", className)} {...props} />
  );
}

function CardDescription({ className, ...props }: ComponentProps<"p">) {
  return (
    <p
      data-slot="card-description"
      className={cn("text-caption text-ink-mute", className)}
      {...props}
    />
  );
}

function CardContent({ className, ...props }: ComponentProps<"div">) {
  return <div data-slot="card-content" className={cn("p-base-xl pt-0", className)} {...props} />;
}

function CardFooter({ className, ...props }: ComponentProps<"div">) {
  return (
    <div
      data-slot="card-footer"
      className={cn("flex items-center p-base-xl pt-0", className)}
      {...props}
    />
  );
}

export { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter };
