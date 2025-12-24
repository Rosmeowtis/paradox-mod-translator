//! 写入模块
//!
//! 将翻译后的内容写入目标目录。

use crate::error::Result;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 写入翻译后的文件
/// 因为 Rust str 本身编码为 UTF-8，所以只需要提前写入 BOM 头即可
pub fn write_translated_file(content: &str, output_path: &Path, create_dirs: bool) -> Result<()> {
    if create_dirs {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_path)?;
    if !content.starts_with("\u{FEFF}") {
        file.write("\u{FEFF}".as_bytes())?;
    }
    file.write(content.as_bytes())?;
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
