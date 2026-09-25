"use client";

/**
 * 模式库浏览器(客户端容器,G 期票 06):承载跨卡的对比勾选状态。
 *
 * 勾选态的「返回后保持」(AC-14)由 URL 承载:compare 页的返回链接带回
 * ?modes=...,页面服务端解析后作为 initialSelected 注入 —— 零本地持久化。
 * 超过 3 个的勾选被拒并提示(RULE-041)。
 */
import { useState } from "react";

import { ModeCard } from "./mode-card";
import type { ModeCard as ModeCardData } from "./state";

export function ModesBrowser({
  items,
  initialSelected,
}: {
  items: ModeCardData[];
  initialSelected: string[];
}) {
  const [selected, setSelected] = useState<string[]>(initialSelected);
  const [hint, setHint] = useState<string | null>(null);

  const toggle = (id: string) => {
    setHint(null);
    setSelected((prev) => {
      if (prev.includes(id)) return prev.filter((x) => x !== id);
      if (prev.length >= 3) {
        setHint("最多对比 3 个,先取消一个再选");
        return prev;
      }
      return [...prev, id];
    });
  };

  const canCompare = selected.length >= 2;

  return (
    <>
      <div className="grid gap-base-md sm:grid-cols-2">
        {items.map((card) => (
          <ModeCard
            key={card.id}
            card={card}
            compareSelected={selected.includes(card.id)}
            onToggleCompare={toggle}
          />
        ))}
      </div>

      {/* 对比操作条:≥2 个才可开始(RULE-041);超限提示就地呈现(ERR-G-03) */}
      <div className="sticky bottom-0 mt-base-md flex flex-wrap items-center gap-base-md rounded-sm border border-hairline bg-canvas-soft px-base-lg py-base-md">
        <span className="text-body-md text-ink">
          已选 <span className="font-mono tabular-nums">{selected.length}</span> / 3
        </span>
        {hint ? (
          <span role="status" className="text-caption text-warning">
            {hint}
          </span>
        ) : null}
        {canCompare ? (
          <a
            href={`/modes/compare?modes=${selected.join(",")}`}
            className="ml-auto inline-flex min-h-11 items-center rounded-sm bg-primary px-base-lg text-button-md font-semibold text-on-primary hover:opacity-90"
          >
            开始对比
          </a>
        ) : (
          <span className="ml-auto text-caption text-ink-mute">勾选 2-3 个模式开始对比</span>
        )}
      </div>
    </>
  );
}
