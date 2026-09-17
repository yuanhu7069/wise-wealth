#!/usr/bin/env python3
"""对比度实测:globals.css 亮/暗两套 token 的实际使用配对(WCAG 2.x)。
阈值:正文/小字 ≥4.5:1,大字(≥24px 或 ≥19px bold) ≥3:1,禁用态 ≥3:1。
跑法:.scratch/stripe-restyle/tmp-contrast-check.py"""

# (名称, 前景, 背景, 阈值, 模式)
PAIRS = [
    # ---- 亮色 ----
    ("L 正文 ink-secondary/canvas", "#273951", "#ffffff", 4.5),
    ("L 正文 ink-secondary/canvas-card", "#273951", "#ffffff", 4.5),
    ("L 正文 ink-secondary/canvas-soft", "#273951", "#f6f9fc", 4.5),
    ("L 标题 ink/canvas-card", "#0d253d", "#ffffff", 4.5),
    ("L 辅助 ink-mute/canvas", "#64748d", "#ffffff", 4.5),
    ("L 辅助 ink-mute/canvas-card", "#64748d", "#ffffff", 4.5),
    ("L 辅助 ink-mute/canvas-soft（已禁用→规则改用 ink-mute-2）", "#64748d", "#f6f9fc", 4.5),
    ("L 辅助 ink-mute-2/canvas-soft", "#61718a", "#f6f9fc", 4.5),
    ("L 禁用 ink-mute-2/canvas-card", "#61718a", "#ffffff", 3.0),
    ("L 链接 action/canvas-card", "#533afd", "#ffffff", 4.5),
    ("L 按钮字 on-primary/primary", "#ffffff", "#533afd", 4.5),
    ("L 按钮字 on-primary/danger-strong", "#ffffff", "#c0553f", 4.5),
    ("L 标签字 primary-press/subdued", "#2e2b8c", "#b9b9f9", 4.5),
    ("L 涨 up/canvas-card", "#c7453c", "#ffffff", 4.5),
    ("L 跌 down/canvas-card", "#27835a", "#ffffff", 4.5),
    ("L 状态 success/canvas-card", "#357a5f", "#ffffff", 4.5),
    ("L warning 图标/canvas（不作小字）", "#b3831f", "#ffffff", 3.0),
    ("L 状态 danger/canvas-card", "#c0553f", "#ffffff", 4.5),
    # ---- 暗色 ----
    ("D 正文 ink-secondary/canvas", "#c3c8de", "#0e1020", 4.5),
    ("D 正文 ink-secondary/canvas-card", "#c3c8de", "#151832", 4.5),
    ("D 正文 ink-secondary/canvas-soft", "#c3c8de", "#1b1f3b", 4.5),
    ("D 标题 ink/canvas-card", "#eef0fb", "#151832", 4.5),
    ("D 辅助 ink-mute/canvas-card", "#848cad", "#151832", 4.5),
    ("D 辅助 ink-mute/canvas-soft", "#848cad", "#1b1f3b", 4.5),
    ("D 禁用 ink-mute-2/canvas-card", "#6b7294", "#151832", 3.0),
    ("D 链接 action/canvas-card", "#b9b9f9", "#151832", 4.5),
    ("D 按钮字 on-primary/primary", "#ffffff", "#533afd", 4.5),
    ("D 按钮字 on-primary/danger-strong", "#ffffff", "#b84e36", 4.5),
    ("D 标签字 深靛/subdued", "#1c1e54", "#b9b9f9", 4.5),
    ("D 涨 up/canvas-card", "#dd5a50", "#151832", 4.5),
    ("D 跌 down/canvas-card", "#3ea46b", "#151832", 4.5),
    ("D 状态 success/canvas-card", "#4fa98a", "#151832", 4.5),
    ("D 状态 warning/canvas-card", "#d9a952", "#151832", 3.0),
    ("D 状态 danger/canvas-card", "#d4694f", "#151832", 4.5),
]


def lin(c: float) -> float:
    return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4


def lum(hexs: str) -> float:
    h = hexs.lstrip("#")
    r, g, b = (int(h[i : i + 2], 16) / 255 for i in (0, 2, 4))
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)


def ratio(fg: str, bg: str) -> float:
    a, b = lum(fg), lum(bg)
    hi, lo = max(a, b), min(a, b)
    return (hi + 0.05) / (lo + 0.05)


fails = 0
for name, fg, bg, th in PAIRS:
    r = ratio(fg, bg)
    ok = r >= th
    fails += 0 if ok else 1
    print(f"{'✓' if ok else '✗'} {r:5.2f}:1 (需≥{th}) {name}")
print(f"\n{'全部通过' if fails == 0 else f'{fails} 项不达标'}")
raise SystemExit(1 if fails else 0)
