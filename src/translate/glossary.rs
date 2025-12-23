//! 术语表模块
//!
//! 加载和管理翻译术语表。每个术语表提供多语言对照。

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crate::error::{Result, TranslationError};

/// 术语表条目
#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    /// 源语言术语
    pub source: String,
    /// 目标语言术语
    pub target: String,
    /// 是否强制使用（不允许翻译）
    pub force: bool,
}

/// 术语表
#[derive(Debug, Clone, Default)]
pub struct Glossary {
    entries: HashMap<String, GlossaryEntry>,
}

impl Glossary {
    /// 从JSON文件加载术语表
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| TranslationError::Translate(
                crate::error::TranslateError::GlossaryError(e.to_string())
            ))?;

        let raw: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| TranslationError::Translate(
                crate::error::TranslateError::GlossaryError(e.to_string())
            ))?;

        let mut entries = HashMap::new();

        if let Some(obj) = raw.as_object() {
            for (key, value) in obj {
                if let Some(target) = value.as_str() {
                    entries.insert(
                        key.clone(),
                        GlossaryEntry {
                            source: key.clone(),
                            target: target.to_string(),
                            force: false,
                        }
                    );
                }
            }
        }

        Ok(Self { entries })
    }

    /// 应用术语表到文本
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (source, entry) in &self.entries {
            // 简单的替换，需要改进为单词边界匹配
            result = result.replace(source, &entry.target);
        }
        result
    }

    /// 获取术语表大小
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}