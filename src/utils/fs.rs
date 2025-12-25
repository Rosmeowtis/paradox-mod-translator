//! 文件系统工具模块

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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

/// 获取用户数据目录路径
///
/// 返回平台特定的用户数据目录：
/// - Unix/Linux: ~/.local/share/pmt/data/
/// - Windows: %APPDATA%\pmt\data\
/// - macOS: ~/.local/share/pmt/data/
pub fn get_user_data_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").map_err(|e| {
            crate::error::TranslationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("APPDATA environment variable not found: {}", e),
            ))
        })?;
        Ok(PathBuf::from(appdata).join("pmt").join("data"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").map_err(|e| {
            crate::error::TranslationError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("HOME environment variable not found: {}", e),
            ))
        })?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("pmt")
            .join("data"))
    }
}

/// 查找数据文件
///
/// 按照以下顺序查找文件：
/// 1. 当前目录下的数据文件：./data/相对路径
/// 2. 用户数据目录下的文件：~/.local/share/pmt/data/相对路径 (Unix) 或 %APPDATA%\pmt\data\相对路径 (Windows)
///
/// 如果文件存在则返回路径，否则返回None
pub fn find_data_file(relative_path: &str) -> Result<Option<PathBuf>> {
    // 1. 尝试当前目录下的data目录
    let current_dir_path = PathBuf::from("data").join(relative_path);
    if current_dir_path.exists() {
        return Ok(Some(current_dir_path));
    }

    // 2. 尝试用户数据目录
    let user_data_dir = get_user_data_dir()?;
    let user_data_path = user_data_dir.join(relative_path);
    if user_data_path.exists() {
        return Ok(Some(user_data_path));
    }

    // 文件不存在
    Ok(None)
}

/// 查找数据文件，如果找不到则返回错误
pub fn find_data_file_or_error(relative_path: &str) -> Result<PathBuf> {
    find_data_file(relative_path)?.ok_or_else(|| {
        let user_data_dir =
            get_user_data_dir().unwrap_or_else(|_| PathBuf::from("[无法获取用户数据目录]"));
        crate::error::TranslationError::FileNotFound(format!(
            "Data file not found: '{}'. Searched in:\n1. ./data/{}\n2. {}/{}",
            relative_path,
            relative_path,
            user_data_dir.display(),
            relative_path
        ))
    })
}
