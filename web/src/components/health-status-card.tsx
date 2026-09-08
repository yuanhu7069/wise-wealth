"use client";

/**
 * 健康状态卡片(P01 唯一交互组件,design-a.md §2)。
 * 页面级五态映射为卡片三形态:骨架屏(默认+加载)/错误(ERR-001/ERR-002)/成功(含空态说明)。
 * 重试交互在本组件内完成(客户端 refetch,经 Server Action 代理走 BFF)。
 */
import { CircleCheck, Compass, Database, ServerOff } from "lucide-react";
import { useEffect, useState, useTransition } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { errorCopyOf } from "@/lib/errors";

/** 卡片状态机:三形态。取数结果由 Server Action 返回(ok/code/version) */
type CardState =
  | { kind: "loading" } // 覆盖页面五态:默认态 + 加载态(骨架 >300ms 才渲染,基线 §7.2)
  | { kind: "error"; errorCode: "ERR_001" | "ERR_002" | string } // 错误态
  | { kind: "success"; version: string }; // 成功态(含空态三要素)

interface Props {
  /** Server 侧取数函数(Server Action):成功返回 ok=true+version,失败返回 ok=false+errorCode */
  fetchHealth: () => Promise<{ ok: boolean; code: string; version?: string }>;
}

const SKELETON_DELAY_MS = 300; // design 基线 §7.2:加载 >300ms 才渲染骨架屏

export function HealthStatusCard({ fetchHealth }: Props) {
  const [state, setState] = useState<CardState>({ kind: "loading" });
  const [showSkeleton, setShowSkeleton] = useState(false); // 300ms 阈值控制
  const [isPending, startTransition] = useTransition();

  const load = () => {
    setState({ kind: "loading" });
    const timer = setTimeout(() => setShowSkeleton(true), SKELETON_DELAY_MS);
    startTransition(async () => {
      try {
        const result = await fetchHealth();
        if (result.ok) {
          setState({ kind: "success", version: result.version ?? "unknown" });
        } else {
          setState({ kind: "error", errorCode: result.code });
        }
      } finally {
        clearTimeout(timer);
        setShowSkeleton(false);
      }
    });
  };

  // 首次挂载即取数
  // biome-ignore lint/correctness/useExhaustiveDependencies: load 依赖稳定的 fetchHealth prop,仅需挂载执行一次
  useEffect(() => {
    load();
  }, []);

  if (state.kind === "loading") {
    if (!showSkeleton && !isPending) {
      // <300ms:不渲染骨架(避免闪烁),留白由卡片容器撑起
      return <Card className="min-h-40" aria-busy="true" aria-label="正在检查服务状态" />;
    }
    // 骨架形状接近真实内容(基线 §7.2:禁止一整块灰矩形)
    return (
      <Card aria-busy="true" aria-label="正在加载服务状态">
        <CardContent className="flex flex-col gap-base-md p-base-xl">
          <div className="flex items-center gap-base-md">
            <Skeleton className="size-6 rounded-sm" />
            <Skeleton className="h-6 w-64" />
          </div>
          <Skeleton className="h-5 w-96 max-w-full" />
          <Skeleton className="h-5 w-72 max-w-full" />
        </CardContent>
      </Card>
    );
  }

  if (state.kind === "error") {
    const copy = errorCopyOf(state.errorCode);
    const isDbError = state.errorCode === "ERR_002";
    const Icon = isDbError ? Database : ServerOff;
    return (
      <Card className="border-destructive/40">
        <CardContent className="flex flex-col gap-base-lg p-base-xl">
          <div className="flex items-start gap-base-md">
            <Icon className="mt-1 size-5 shrink-0 text-destructive" aria-hidden="true" />
            <div className="flex flex-col gap-base-xs">
              <p className="text-body font-medium text-text-title">{copy.title}</p>
              <p className="text-body text-text-body">{copy.description}</p>
              <p className="text-label text-text-aux">错误码:{state.errorCode}</p>
            </div>
          </div>
          <div>
            <Button variant="ghost" size="sm" onClick={load} disabled={isPending}>
              重试
            </Button>
          </div>
        </CardContent>
      </Card>
    );
  }

  // 成功态(含空态三要素:Compass 图标 + 人话说明 + 明确引导——design 基线 §7.3)
  return (
    <Card>
      <CardContent className="flex flex-col gap-base-lg p-base-xl">
        <div className="flex items-center gap-base-md">
          <CircleCheck className="size-5 shrink-0 text-success" aria-hidden="true" />
          <p className="text-body text-text-body">
            服务在线 · 数据库已连接 ·{" "}
            <span className="text-data tabular-nums text-text-title">v{state.version}</span>
          </p>
        </div>
        <div className="flex items-start gap-base-md rounded-md bg-primary-bg p-base-lg">
          <Compass className="mt-0.5 size-5 shrink-0 text-text-aux" aria-hidden="true" />
          <div className="flex flex-col gap-base-xs">
            <p className="text-body text-text-body">
              服务在线,暂无业务模块——A 期是工程骨架期,各页面共用这套状态卡片与 Token 体系。
            </p>
            <p className="text-aux text-text-aux">
              业务功能将随 B 期(规则引擎)起逐步提供,详见 docs/智策理财_PRD_V1.1.md。
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
