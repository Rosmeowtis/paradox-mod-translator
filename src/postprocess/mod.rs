//! 后处理模块
//!
//! 负责合并翻译后的切片，写入目标目录，并清理临时文件。

mod cleanup;
mod merger;
mod writer;

pub use cleanup::*;
pub use merger::*;
pub use writer::*;