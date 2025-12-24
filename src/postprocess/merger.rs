//! 合并模块
//!
//! 合并翻译后的切片为完整的文件。

use crate::error::{Result, TranslationError};

/// 翻译切片
pub struct TranslationSlice {
    /// 切片内容
    pub content: String,
    /// 起始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
}

/// 合并翻译切片并恢复缩进
pub fn merge_slices(slices: Vec<TranslationSlice>) -> Result<String> {
    if slices.is_empty() {
        return Err(TranslationError::Postprocess(
            crate::error::PostprocessError::InconsistentSlices,
        ));
    }

    // 按起始行排序
    let mut sorted_slices = slices;
    sorted_slices.sort_by_key(|s| s.start_line);

    // 检查连续性
    for i in 1..sorted_slices.len() {
        if sorted_slices[i].start_line != sorted_slices[i - 1].end_line + 1 {
            return Err(TranslationError::Postprocess(
                crate::error::PostprocessError::MergeFailed(format!(
                    "Slices are not contiguous: {} != {} + 1",
                    sorted_slices[i].start_line,
                    sorted_slices[i - 1].end_line
                )),
            ));
        }
    }

    // 合并内容并增加两级缩进
    let mut lines = Vec::new();
    for slice in sorted_slices {
        lines.extend(slice.content.lines().map(|s| format!("  {}", s)));
    }

    Ok(lines.join("\n"))
}

/// 从YAML内容重建完整文件
pub fn reconstruct_yaml_file(slices: Vec<TranslationSlice>, target_lang: &str) -> Result<String> {
    let merged = merge_slices(slices)?;

    let target_lang_header = format!("l_{}:\n", target_lang);
    let mut result = String::new();
    // 确保有文件头
    result.push_str(&target_lang_header);
    result.push_str(&merged);
    Ok(result)
}
