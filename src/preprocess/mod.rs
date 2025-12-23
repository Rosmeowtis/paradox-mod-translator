//! 预处理模块
//!
//! 负责清洗和整理原始本地化文件，修复YAML格式问题，并将大文件切片。

mod normalizer;
mod splitter;
mod yaml_fixer;

pub use normalizer::*;
pub use splitter::*;
pub use yaml_fixer::*;