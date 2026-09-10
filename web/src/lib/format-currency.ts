/**
 * 金额与「约 N 个月」的统一格式化(design-v2 §3.2)。
 *
 * 全站只此一份:金额一律「¥ 千分位 + 2 位小数」、月数一律一位小数 ——
 * 同一种数字在首页与方案页看起来不一样,是产品级的不一致(基线 §6.1「数字」行)。
 *
 * 入参是**整数分**(ADR-004 全链路整数分);本模块是唯一把分变成展示字符串的地方。
 * 手写而不用 `Intl.NumberFormat("zh-CN", {currency:"CNY"})`:后者的符号与分隔符随 ICU
 * 数据版本变化(Node 与浏览器可能不同),而金额展示不允许出现环境相关的差异。
 */

/** 分 → 「¥4,500.00」。负号在 ¥ 之前,千分位固定用逗号。 */
export function formatCurrency(cents: number): string {
  const safe = Number.isFinite(cents) ? Math.trunc(cents) : 0;
  const sign = safe < 0 ? "-" : "";
  const abs = Math.abs(safe);
  const yuan = Math.floor(abs / 100);
  const fen = abs % 100;
  return `${sign}¥${grouped(yuan)}.${String(fen).padStart(2, "0")}`;
}

/**
 * 应急金距离感文案的数值部分(design-v2 §3.1「应急状态句」):×10 的整数 → 一位小数。
 * 0.4 个月说「还差 0.4 个月」;已达标时用同一个函数说「已覆盖(约 18.5 个月)」。
 */
export function formatMonthsTenths(tenths: number): string {
  const safe = Number.isFinite(tenths) ? Math.trunc(tenths) : 0;
  const sign = safe < 0 ? "-" : "";
  const abs = Math.abs(safe);
  return `${sign}${Math.floor(abs / 10)}.${abs % 10}`;
}

function grouped(value: number): string {
  const digits = String(value);
  let out = "";
  for (let i = 0; i < digits.length; i += 1) {
    if (i > 0 && (digits.length - i) % 3 === 0) out += ",";
    out += digits[i];
  }
  return out;
}
