//! 清理模块
//!
//! 清理临时文件和中间文件。

use crate::error::Result;
use std::path::Path;

/// 清理临时文件
pub fn cleanup_temp_files(temp_dir: &Path) -> Result<()> {
    if temp_dir.exists() {
        std::fs::remove_dir_all(temp_dir)?;
    }
    Ok(())
}

/// 清理特定扩展名的文件
pub fn cleanup_files_by_extension(dir: &Path, extension: &str) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == extension {
                    std::fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(())
}
