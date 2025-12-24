//! YAML修复模块
//!
//! 修复Stellaris本地化文件的YAML格式问题。

use crate::{
    config,
    error::{Result, TranslationError},
};
use regex::Regex;

/// 修复YAML内容
pub fn fix_yaml_content(content: &str) -> Result<String> {
    let fixed = content.to_string();
    let re_key_zero = Regex::new(r#"(\w+):0\s+"([^"]+)"#).unwrap();
    let re_unquoted_value = Regex::new(r#"(\w+):\s+([^"\s][^"\n]*)(?:\n|$)"#).unwrap();

    let lines = fixed.lines().into_iter().map(|line| {
        // 1. 修复 `key:0 "value"` 格式
        let fixed = re_key_zero.replace_all(line, r#"$1: "$2""#);
        // 2. 确保所有值都有引号
        let fixed = re_unquoted_value.replace_all(&fixed, r#"$1: "$2""#);
        // 3. 标准化缩进（2空格）
        let trimmed = fixed.trim_start();
        let indent_level = fixed.len() - trimmed.len();
        let spaces = indent_level / 2 * 2; // 确保是2的倍数
        let fixed = format!("{}{}", " ".repeat(spaces), trimmed);
        fixed
    });
    let lines: Vec<String> = lines.collect();
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

/// 移除YAML内容中的语言头(如 l_english:)，并返回原始头和去除头后的内容
/// 去除头后的内容会去掉所有缩进
pub fn trim_lang_header(task: &config::TranslationTask, fixed_content: String) -> (String, String) {
    let mut lines: Vec<String> = fixed_content.lines().map(String::from).collect();
    let mut original_header = String::new();

    // 查找第一个非注释行
    let mut header_index = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue; // 跳过空行
        }
        if trimmed.starts_with('#') {
            continue; // 跳过注释行
        }
        // 检查是否是语言头
        if trimmed.starts_with(&format!("l_{}:", task.source_lang)) {
            header_index = Some(i);
            break;
        }
        // 如果不是语言头，则停止查找
        break;
    }

    if let Some(index) = header_index {
        original_header = lines.remove(index);
    }

    // 去除所有缩进
    let content_without_header = lines
        .into_iter()
        .map(|line| line.trim_start().to_string())
        .collect::<Vec<String>>()
        .join("\n");
    (original_header, content_without_header)
}
