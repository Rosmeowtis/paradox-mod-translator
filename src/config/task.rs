use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::ClientSettings;

/// 从TOML文件加载的翻译任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationTask {
    /// 源语言代码（例如："english"）
    pub source_lang: String,

    /// 目标语言代码列表（例如：["simp_chinese"]）
    pub target_langs: Vec<String>,

    /// 使用的术语表名称（不带.json扩展名）
    pub glossaries: Vec<String>,

    /// 本地化文件目录路径
    pub localisation_dir: PathBuf,
}

/// 完整的任务配置文件结构
#[derive(Debug, Deserialize)]
struct TaskFileConfig {
    /// 大模型客户端设置（可选，使用默认值）
    #[serde(default)]
    client_settings: ClientSettings,
    /// 翻译任务列表
    task: Vec<TranslationTask>,
}

impl TaskFileConfig {
    /// 从TOML文件加载配置
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, crate::error::ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::ConfigError::InvalidPath(e.to_string()))?;

        let config: TaskFileConfig = toml::from_str(&content)
            .map_err(crate::error::ConfigError::TomlParse)?;

        // 验证客户端设置
        config.client_settings.validate()?;

        // 验证每个任务
        for task in &config.task {
            task.validate()?;
        }

        Ok(config)
    }
}

impl TranslationTask {
    /// 从TOML文件加载翻译任务（返回第一个任务和客户端设置）
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<(ClientSettings, Self), crate::error::ConfigError> {
        let config = TaskFileConfig::from_file(path)?;

        // 目前只取第一个任务
        // TODO: 支持单个文件中的多个任务
        config.task.into_iter().next()
            .map(|task| (config.client_settings, task))
            .ok_or_else(|| crate::error::ConfigError::MissingField("配置文件中未找到任务".to_string()))
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), crate::error::ConfigError> {
        if self.source_lang.is_empty() {
            return Err(crate::error::ConfigError::MissingField("source_lang".to_string()));
        }

        if self.target_langs.is_empty() {
            return Err(crate::error::ConfigError::MissingField("target_lang".to_string()));
        }

        if !self.localisation_dir.exists() {
            return Err(crate::error::ConfigError::InvalidPath(
                format!("本地化目录不存在: {:?}", self.localisation_dir)
            ));
        }

        // 检查源语言目录是否存在
        let source_dir = self.localisation_dir.join(&self.source_lang);
        if !source_dir.exists() {
            return Err(crate::error::ConfigError::InvalidPath(
                format!("源语言目录不存在: {:?}", source_dir)
            ));
        }

        Ok(())
    }

    /// 获取源语言目录路径
    pub fn source_dir(&self) -> PathBuf {
        self.localisation_dir.join(&self.source_lang)
    }

    /// 获取特定目标语言的目标目录路径
    pub fn target_dir(&self, target_lang: &str) -> PathBuf {
        self.localisation_dir.join(target_lang).join("replace")
    }
}