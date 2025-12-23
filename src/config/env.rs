//! 环境变量和API密钥管理模块
//!
//! 安全地加载API密钥，优先从环境变量读取，支持`.env`文件。

use std::env;

/// 从环境变量加载OpenAI兼容的API密钥
///
/// 搜索顺序：
/// 1. `OPENAI_API_KEY` 环境变量
/// 2. `.env` 文件中的 `OPENAI_API_KEY`
///
/// # 错误
/// 如果未找到API密钥，返回`MissingEnvVar`错误。
pub fn load_openai_api_key() -> Result<String, crate::error::TranslationError> {
    // 首先尝试加载.env文件（如果存在）
    let _ = dotenvy::dotenv();

    env::var("OPENAI_API_KEY")
        .map_err(|_| crate::error::TranslationError::MissingEnvVar(
            "OPENAI_API_KEY environment variable is required".to_string()
        ))
}

/// 检查API密钥是否已设置（不实际加载值）
pub fn has_api_key() -> bool {
    let _ = dotenvy::dotenv();
    env::var("OPENAI_API_KEY").is_ok()
}