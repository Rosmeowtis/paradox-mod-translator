//! 翻译模块
//!
//! 负责与大模型API交互，加载术语表，执行翻译并验证结果。

mod api;
mod batcher;
mod glossary;
mod splitter;
mod translator;
mod validator;

pub use api::*;
pub use batcher::*;
pub use glossary::*;
pub use splitter::*;
pub use translator::*;
pub use validator::*;
