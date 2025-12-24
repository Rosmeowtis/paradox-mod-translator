//! 写入模块
//!
//! 将翻译后的内容写入目标目录。

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// 写入翻译后的文件
pub fn write_translated_file(content: &str, output_path: &Path, create_dirs: bool) -> Result<()> {
    if create_dirs {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, content)?;
    Ok(())
}

/// 批量写入翻译文件
pub fn write_translated_files(
    files: Vec<(PathBuf, String)>,
    base_output_dir: &Path,
    create_dirs: bool,
) -> Result<usize> {
    let mut success_count = 0;

    for (relative_path, content) in files {
        let output_path = base_output_dir.join(&relative_path);

        if let Err(e) = write_translated_file(&content, &output_path, create_dirs) {
            log::error!("Failed to write file {:?}: {}", output_path, e);
        } else {
            log::info!("Successfully wrote file: {:?}", output_path);
            success_count += 1;
        }
    }

    Ok(success_count)
}
