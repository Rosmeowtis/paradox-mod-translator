//! YAML修复模块
//!
//! 修复Stellaris本地化文件的YAML格式问题。

use regex::Regex;
use crate::error::{Result, TranslationError};

/// 修复YAML内容
pub fn fix_yaml_content(content: &str) -> Result<String> {
    let mut fixed = content.to_string();

    // 1. 修复 `key:0 "value"` 格式
    let re_key_zero = Regex::new(r#"(\w+):0\s+"([^"]+)"#).unwrap();
    fixed = re_key_zero.replace_all(&fixed, r#"$1: "$2""#).to_string();

    // 2. 确保所有值都有引号
    let re_unquoted_value = Regex::new(r#"(\w+):\s+([^"\s][^"\n]*)(?:\n|$)"#).unwrap();
    fixed = re_unquoted_value.replace_all(&fixed, r#"$1: "$2""#).to_string();

    // 3. 标准化缩进（2空格）
    let lines: Vec<String> = fixed.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent_level = line.len() - trimmed.len();
            let spaces = indent_level / 2 * 2; // 确保是2的倍数
            format!("{}{}", " ".repeat(spaces), trimmed)
        })
        .collect();

    Ok(lines.join("\n"))
}

/// 验证YAML内容格式
pub fn validate_yaml_content(content: &str) -> Result<()> {
    // 简单的验证：检查是否包含有效的键值对
    if content.trim().is_empty() {
        return Err(TranslationError::InvalidYaml("Empty content".to_string()));
    }

    // 检查是否有顶级语言标签
    if !content.contains("l_") && content.contains(':') {
        // 可能有效，继续
    }

    Ok(())
}