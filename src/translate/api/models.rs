//! API请求和响应数据结构

use serde::{Deserialize, Serialize};

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色：system, user, assistant
    pub role: String,
    /// 消息内容
    pub content: String,
}

/// 聊天补全请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// 模型名称
    pub model: String,
    /// 消息列表
    pub messages: Vec<ChatMessage>,
    /// 温度参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// 最大token数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// 是否流式响应
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// 聊天补全响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// 响应ID
    pub id: String,
    /// 对象类型
    pub object: String,
    /// 创建时间戳
    pub created: u64,
    /// 模型名称
    pub model: String,
    /// 选择列表
    pub choices: Vec<ChatChoice>,
    /// 使用情况统计
    pub usage: UsageStats,
}

/// 聊天选择
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// 索引
    pub index: u32,
    /// 消息
    pub message: ChatMessage,
    /// 完成原因
    pub finish_reason: String,
}

/// 使用情况统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// 提示token数
    pub prompt_tokens: u32,
    /// 补全token数
    pub completion_tokens: u32,
    /// 总token数
    pub total_tokens: u32,
}

/// 创建系统消息
pub fn system_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "system".to_string(),
        content,
    }
}

/// 创建用户消息
pub fn user_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "user".to_string(),
        content,
    }
}

/// 创建助手消息
pub fn assistant_message(content: String) -> ChatMessage {
    ChatMessage {
        role: "assistant".to_string(),
        content,
    }
}