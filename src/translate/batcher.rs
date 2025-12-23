//! 批处理模块
//!
//! 管理翻译任务的批处理和并发控制。

use crate::error::Result;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// 批处理管理器
pub struct TranslationBatcher {
    max_concurrent: usize,
}

impl TranslationBatcher {
    /// 创建新的批处理器
    pub fn new(max_concurrent: usize) -> Self {
        Self { max_concurrent }
    }

    /// 批量处理翻译任务
    pub async fn process_batch<F, T>(&self, items: Vec<T>, process_fn: F) -> Result<Vec<T>>
    where
        F: Fn(T) -> Result<T> + Send + Sync + 'static,
        T: Send + 'static + Clone,
    {
        if items.is_empty() {
            return Ok(vec![]);
        }

        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let process_fn = Arc::new(process_fn);
        let mut tasks = Vec::new();

        for item in items {
            let permit = semaphore.clone().acquire_owned().await.map_err(|e| {
                crate::error::TranslationError::Translate(
                    crate::error::TranslateError::ValidationFailed(format!(
                        "Failed to acquire semaphore: {}",
                        e
                    )),
                )
            })?;
            let process_fn = process_fn.clone();

            let task = tokio::spawn(async move {
                let result = process_fn(item);
                drop(permit); // 释放信号量许可
                result
            });

            tasks.push(task);
        }

        // 等待所有任务完成
        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(Ok(item)) => results.push(item),
                Ok(Err(e)) => return Err(e),
                Err(e) => {
                    return Err(crate::error::TranslationError::from(
                        crate::error::TranslateError::ValidationFailed(e.to_string()),
                    ));
                }
            }
        }

        Ok(results)
    }
}
