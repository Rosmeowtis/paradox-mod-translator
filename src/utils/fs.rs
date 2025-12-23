//! 文件系统工具模块

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::error::Result;

/// 递归查找所有YAML文件
pub fn find_yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    files.push(path.to_path_buf());
                }
            }
        }
    }

    Ok(files)
}

/// 读取文件内容，自动处理BOM
pub fn read_file_with_bom(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;

    // 移除UTF-8 BOM
    let content = if content.starts_with('\u{feff}') {
        &content[3..]
    } else {
        &content
    };

    Ok(content.to_string())
}

/// 安全创建目录（如果不存在）
pub fn ensure_dir_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 获取文件相对路径
pub fn get_relative_path(base: &Path, full_path: &Path) -> Option<PathBuf> {
    full_path.strip_prefix(base).ok().map(|p| p.to_path_buf())
}

/// 计算文件大小（字符数）
pub fn get_file_size_chars(path: &Path) -> Result<usize> {
    let content = fs::read_to_string(path)?;
    Ok(content.chars().count())
}