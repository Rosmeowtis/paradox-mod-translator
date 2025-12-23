//! 翻译任务配置模块
//!
//! 处理任务配置文件的加载和验证（TOML格式）。

mod client_settings;
mod env;
mod task;

pub use client_settings::*;
pub use env::*;
pub use task::*;