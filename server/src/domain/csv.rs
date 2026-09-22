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

/// 是否为「纯数字」单元格(可选负号 + 数字 + 至多一位小数点)。
/// 这类值写进 Excel 也只会是数字,不存在公式执行 —— 绕过转义,
/// 保住 Excel 里的数值类型(否则 `-123.45` 会变 `'-123.45` 文本,破坏二次分析)。
fn is_plain_number(s: &str) -> bool {
    let digits = s.strip_prefix('-').unwrap_or(s);
    if digits.is_empty() {
        return false;
    }
    let (int_part, frac_part) = match digits.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (digits, None),
    };
    !int_part.is_empty()
        && int_part.chars().all(|c| c.is_ascii_digit())
        && frac_part.is_none_or(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()))
}

/// 单元格出口:纯数字原样,其余走 [`escape_field`]。
fn emit_cell(s: &str) -> String {
    if is_plain_number(s) {
        s.to_string()
    } else {
        escape_field(s)
    }
}

/// 整表渲染(RULE-030):UTF-8 BOM 打头(Excel 中文兼容),行以 CRLF 结尾
/// (RFC 4180),非数字字段一律走 [`escape_field`]。
pub fn render_csv(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::from("\u{feff}");
    let mut emit_line = |fields: Vec<String>| {
        let cells: Vec<String> = fields.into_iter().map(|f| emit_cell(&f)).collect();
        out.push_str(&cells.join(","));
        out.push_str("\r\n");
    };
    emit_line(header.iter().map(|h| h.to_string()).collect());
    for row in rows {
        emit_line(row.clone());
    }
    out
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

    #[test]
    fn 整表渲染带_bom_与_crlf_且文本字段全转义() {
        let csv = render_csv(
            &["月份", "桶名", "余额(元)"],
            &[vec![
                "2026-09".into(),
                "=危险".into(),
                yuan_string(-12345),
            ]],
        );
        assert!(csv.starts_with('\u{feff}'), "必须以 BOM 打头");
        assert!(csv.ends_with("\r\n"), "行尾必须是 CRLF");
        assert!(
            csv.contains(",'=危险,"),
            "公式前缀文本必须被拦截: {csv}"
        );
        assert!(
            csv.contains(",-123.45\r\n"),
            "纯数字保持数值类型,不加转义前缀: {csv}"
        );
        assert_eq!(csv.lines().count(), 2, "表头 + 一行数据");
    }

    #[test]
    fn 纯数字判定不放过冒牌货() {
        assert!(is_plain_number("0"));
        assert!(is_plain_number("-123.45"));
        assert!(is_plain_number("123"));
        assert!(is_plain_number("0.05"));
        assert!(!is_plain_number("-12.3.4"));
        assert!(!is_plain_number("12."));
        assert!(!is_plain_number("2026-09"), "日期不是纯数字");
        assert!(!is_plain_number(""));
        assert!(!is_plain_number("-"));
        assert!(!is_plain_number("12a"));
        assert!(!is_plain_number("=1"));
    }

    #[test]
    fn 空表只剩表头一行() {
        let csv = render_csv(&["月份"], &[]);
        assert_eq!(csv, "\u{feff}月份\r\n");
    }
}
