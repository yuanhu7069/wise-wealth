#!/usr/bin/env bash
# Token 门禁(RULE-006):src 下(除 globals.css)禁止裸色值与 px 字面量。
# - 色值:hex(#xxx)、rgb/rgba(、hsl(、oklch(、常见 CSS 命名色
# - 尺寸:数字+px(字体/间距必须走 Token 工具类)
# 用法:scripts/check-tokens.sh  —— 命中即退出码 1
set -euo pipefail
cd "$(dirname "$0")/.."

hits=$(grep -rnE --include='*.tsx' --include='*.ts' \
  '#[0-9a-fA-F]{3,8}\b|rgba?\(|hsla?\(|oklch\(|\b(white|black|red|blue|green|gray|grey|zinc|slate)\b(-[0-9]{2,3})?|\b[0-9]+(\.[0-9]+)?px\b' \
  src/ 2>/dev/null | grep -v 'src/app/globals.css' || true)

if [ -n "$hits" ]; then
  echo "✗ Token 门禁失败:发现裸色值/px(RULE-006):"
  echo "$hits"
  exit 1
fi
echo "✓ Token 门禁通过:src 下无裸色值/px(globals.css 除外)"
