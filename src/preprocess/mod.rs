//! 预处理模块
//!
//! 负责清洗和整理原始本地化文件，修复YAML格式问题，并将大文件切片。

mod file_prepare;
mod normalizer;
mod yaml_fixer;

pub use file_prepare::*;
pub use normalizer::*;
pub use yaml_fixer::*;
