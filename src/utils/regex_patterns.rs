//! 正则表达式模式
//!
//! 预编译的正则表达式，用于YAML修复和验证。

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// 匹配 `key:0 "value"` 格式
    pub static ref KEY_ZERO_PATTERN: Regex = Regex::new(r#"(\w+):0\s+"([^"]+)"#).unwrap();

    /// 匹配未加引号的值
    pub static ref UNQUOTED_VALUE_PATTERN: Regex = Regex::new(r#"(\w+):\s+([^"\s][^"\n]*)(?:\n|$)"#).unwrap();

    /// 匹配图标标记 £...£
    pub static ref ICON_PATTERN: Regex = Regex::new(r#"£[^£]+£"#).unwrap();

    /// 匹配变量标记 $...$
    pub static ref VARIABLE_PATTERN: Regex = Regex::new(r#"\$[^$]+\$"#).unwrap();

    /// 匹配颜色标记 §...§
    pub static ref COLOR_PATTERN: Regex = Regex::new(r#"§[^§]+§"#).unwrap();

    /// 匹配YAML键（用于提取）
    pub static ref YAML_KEY_PATTERN: Regex = Regex::new(r#"^\s*(\w+):"#).unwrap();

    /// 匹配YAML注释
    pub static ref YAML_COMMENT_PATTERN: Regex = Regex::new(r#"#.*$"#).unwrap();

    /// 匹配换行符（用于标准化）
    pub static ref NEWLINE_PATTERN: Regex = Regex::new(r#"\r\n|\r|\n"#).unwrap();
}

/// 修复YAML中的键零格式
pub fn fix_key_zero_format(text: &str) -> String {
    KEY_ZERO_PATTERN.replace_all(text, r#"$1: "$2""#).to_string()
}

/// 为未加引号的值添加引号
pub fn quote_unquoted_values(text: &str) -> String {
    UNQUOTED_VALUE_PATTERN.replace_all(text, r#"$1: "$2""#).to_string()
}

/// 提取所有特殊标记
pub fn extract_all_markers(text: &str) -> Vec<String> {
    let mut markers = Vec::new();

    markers.extend(ICON_PATTERN.find_iter(text).map(|m| m.as_str().to_string()));
    markers.extend(VARIABLE_PATTERN.find_iter(text).map(|m| m.as_str().to_string()));
    markers.extend(COLOR_PATTERN.find_iter(text).map(|m| m.as_str().to_string()));

    markers
}

/// 检查文本是否包含特殊标记
pub fn contains_markers(text: &str) -> bool {
    ICON_PATTERN.is_match(text) ||
    VARIABLE_PATTERN.is_match(text) ||
    COLOR_PATTERN.is_match(text)
}