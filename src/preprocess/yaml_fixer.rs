//! YAML修复模块
//!
//! 修复Stellaris本地化文件的YAML格式问题。

use crate::error::Result;
use regex::Regex;

/// 修复YAML内容
pub fn fix_yaml_content(content: &str) -> Result<String> {
    let fixed = content.to_string();
    // 都是单行处理
    // 处理如 key:0 value 的形式，不假定 value 存在或有完整的引号
    let re_key_zero = Regex::new(r#"^(\w+):\d+\s+(.*)$"#).unwrap();
    // 处理 key: value 的形式，value 可能有两个引号，或只有一侧有引号，或没有引号
    let re_unquoted_value = Regex::new(r#"^(\w+):\s+"?([^"]*)"?$"#).unwrap();

    let lines = fixed.lines().into_iter().map(|line| {
        // 0. 跳过空行和注释行
        if line.trim().is_empty() {
            return "".into();
        }
        if line.trim_start().starts_with('#') {
            return line.into();
        }
        // 1. 修复 `key:0 "value"` 格式
        let fixed = re_key_zero.replace(line, r#"$1: $2"#);
        // 2. 确保所有值都有引号
        let fixed = re_unquoted_value.replace(&fixed, r#"$1: "$2""#);
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

/// 移除YAML内容中的语言头(如 l_english:)，并返回原始头和去除头后的内容
/// 去除头后的内容会去掉所有缩进
pub fn trim_lang_header(lang: &str, fixed_content: &str) -> (String, String) {
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
        if trimmed.starts_with(&format!("l_{}:", lang)) {
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
