//! CSV 导出的纯格式化(ADR-E-004:手拼转义,不引 csv crate)。
//!
//! 转义规则两条:
//! 1. **公式注入防护**(OWASP CSV Injection):文本以 `=` `+` `-` `@` 或制表符开头时
//!    前缀单引号 —— Excel 会把 `=HYPERLINK(…)` 当公式执行,这是导出功能的经典攻击面;
//! 2. **RFC 4180**:含逗号 / 引号 / 换行的字段加引号包裹,内部引号倍增。
//!
//! **数值列不走转义**,直接写 `yuan_string` 的产物:它是校验过的 i64 分,
//! 不是任意文本,加前缀反而让 Excel 里的数字变成文本、破坏二次分析。

/// 文本字段转义:公式前缀防护 + RFC 4180 引号规则。
pub fn escape_field(raw: &str) -> String {
    let needs_quotes = raw
        .chars()
        .any(|c| matches!(c, ',' | '"' | '\n' | '\r'));
    let mut out = String::with_capacity(raw.len() + 2);
    // 公式前缀防护:单引号让 Excel 把后续内容按文本处理(前缀本身不显示)
    if let Some(first) = raw.chars().next()
        && matches!(first, '=' | '+' | '-' | '@' | '\t')
    {
        out.push('\'');
    }
    if needs_quotes {
        out.push('"');
        for c in raw.chars() {
            if c == '"' {
                out.push_str("\"\"");
            } else {
                out.push(c);
            }
        }
        out.push('"');
    } else {
        out.push_str(raw);
    }
    out
}

/// 金额(分)→ 元字符串,固定两位小数;负数带负号(透支是真实状态,RULE-022)。
pub fn yuan_string(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{sign}{}.{:02}", abs / 100, abs % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 平文本原样通过() {
        assert_eq!(escape_field("2026-09"), "2026-09");
        assert_eq!(escape_field("工资账户"), "工资账户");
        assert_eq!(escape_field(""), "");
    }

    #[test]
    fn 公式前缀字符被单引号拦截() {
        // 不含逗号/引号/换行的公式样本:只加前缀,不加引号包裹
        assert_eq!(escape_field("=SUM(A1)"), "'=SUM(A1)");
        assert_eq!(escape_field("@sum"), "'@sum");
        assert_eq!(escape_field("+1+cmd|' /C calc'!A0"), "'+1+cmd|' /C calc'!A0");
        // 制表符开头同样按可执行前缀处理
        assert_eq!(escape_field("\tx"), "'\tx");
    }

    #[test]
    fn 含逗号引号换行的字段加引号且内部引号倍增() {
        assert_eq!(escape_field("a,b"), "\"a,b\"");
        assert_eq!(escape_field("说\"实话\""), "\"说\"\"实话\"\"\"");
        assert_eq!(escape_field("两\n行"), "\"两\n行\"");
    }

    #[test]
    fn 前缀防护与引号包裹可以叠加() {
        // 以 '-' 开头且含逗号:先加前缀,再整体加引号
        assert_eq!(escape_field("-a,b"), "'\"-a,b\"");
    }

    #[test]
    fn 金额两位小数含负数与零() {
        assert_eq!(yuan_string(0), "0.00");
        assert_eq!(yuan_string(12345), "123.45");
        assert_eq!(yuan_string(5), "0.05");
        assert_eq!(yuan_string(-5), "-0.05");
        assert_eq!(yuan_string(-12345), "-123.45");
        assert_eq!(yuan_string(-100), "-1.00");
    }
}
