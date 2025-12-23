use serde::{Deserialize, Serialize};

/// 大模型客户端设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSettings {
    /// API基础URL（OpenAI兼容格式）
    #[serde(default = "default_api_base")]
    pub api_base: String,

    /// 模型名称
    #[serde(default = "default_model")]
    pub model: String,

    /// 温度参数（0.0-2.0）
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// 请求超时时间（秒）
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// 最大重试次数
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// 最大输出token数
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// 每次请求的最大文本长度（字符数，用于切片）
    #[serde(default = "default_max_chunk_size")]
    pub max_chunk_size: usize,

    /// 是否启用流式响应
    #[serde(default)]
    pub stream: bool,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            api_base: default_api_base(),
            model: default_model(),
            temperature: default_temperature(),
            timeout_secs: default_timeout(),
            max_retries: default_max_retries(),
            max_tokens: default_max_tokens(),
            max_chunk_size: default_max_chunk_size(),
            stream: false,
        }
    }
}

fn default_api_base() -> String {
    "https://api.deepseek.com".to_string()
}

fn default_model() -> String {
    "deepseek-reasoner".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

// 大模型翻译可能耗时长久，默认超时设置为10分钟
fn default_timeout() -> u64 {
    600
}

fn default_max_retries() -> u32 {
    3
}

fn default_max_tokens() -> u32 {
    2000
}

fn default_max_chunk_size() -> usize {
    4000 // 大约1000个token的保守估计
}

impl ClientSettings {
    /// 验证设置是否有效
    pub fn validate(&self) -> Result<(), crate::error::ConfigError> {
        if self.temperature < 0.0 || self.temperature > 2.0 {
            return Err(crate::error::ConfigError::MissingField(
                "temperature must be between 0.0 and 2.0".to_string(),
            ));
        }

        if self.timeout_secs == 0 {
            return Err(crate::error::ConfigError::MissingField(
                "timeout_secs must be greater than 0".to_string(),
            ));
        }

        if self.max_chunk_size < 100 {
            return Err(crate::error::ConfigError::MissingField(
                "max_chunk_size must be at least 100 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// 获取完整的API端点URL
    pub fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.api_base)
    }
}
