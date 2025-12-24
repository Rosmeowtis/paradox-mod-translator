//! 文件切片模块
//!
//! 将大文件分割为适合大模型上下文大小的切片。

use crate::error::Result;
use crate::utils::estimate_mixed_tokens;

/// 文件切片
pub struct FileChunk {
    /// 切片内容
    pub content: String,
    /// 在原文件中的起始位置
    pub start_line: usize,
    /// 在原文件中的结束位置
    pub end_line: usize,
    /// 目标文件名
    pub target_filename: String,
}

/// 将YAML内容分割为多个切片
pub fn split_yaml_content(
    target_filename: &str,
    content: &str,
    max_chunk_size: usize,
) -> Result<Vec<FileChunk>> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Ok(vec![]);
    }

    let mut chunks = Vec::new();
    let mut current_chunk_lines = Vec::new();
    let mut current_token_count = 0;
    let mut start_line = 1;

    for (i, line) in lines.iter().enumerate() {
        let line_number = i + 1;
        let line_token_count = estimate_mixed_tokens(line);

        // 如果当前行会使token数超过限制，且当前切片不为空，则结束当前切片
        if !current_chunk_lines.is_empty()
            && current_token_count + line_token_count > max_chunk_size
        {
            let end_line = line_number - 1;
            chunks.push(FileChunk {
                content: current_chunk_lines.join("\n"),
                start_line,
                end_line,
                target_filename: target_filename.to_string(),
            });

            // 开始新切片
            current_chunk_lines = vec![*line];
            current_token_count = line_token_count;
            start_line = line_number;
        } else {
            // 添加到当前切片
            current_chunk_lines.push(*line);
            current_token_count += line_token_count;
        }
    }

    // 添加最后一个切片
    if !current_chunk_lines.is_empty() {
        let end_line = lines.len();
        chunks.push(FileChunk {
            content: current_chunk_lines.join("\n"),
            start_line,
            end_line,
            target_filename: target_filename.to_string(),
        });
    }

    // 如果只有一个切片且未超过限制，直接返回
    Ok(chunks)
}
