//! 翻译器模块
//!
//! 集成API客户端、术语表和提示词模板，执行翻译任务。

use crate::config::ClientSettings;
use crate::error::{Result, TranslationError};
use crate::postprocess::TranslationSlice;
use crate::translate::FileChunk;
use crate::translate::api::{ApiClient, system_message, user_message};
use crate::translate::glossary::Glossary;
use crate::translate::validator::FormatValidator;
use crate::utils::{estimate_mixed_tokens, find_data_file_or_error};
use std::fs;

/// 翻译器
pub struct Translator {
    api_client: ApiClient,
    glossary: Glossary,
    validator: FormatValidator,
}

impl Translator {
    /// 创建新的翻译器
    pub fn new(api_client: ApiClient, glossaries: Glossary) -> Self {
        Self {
            api_client,
            glossary: glossaries,
            validator: FormatValidator::new(),
        }
    }

    /// 从设置创建翻译器
    pub fn from_settings(client_settings: ClientSettings, glossary: Glossary) -> Result<Self> {
        let api_key = crate::config::load_openai_api_key()?;
        let api_client = ApiClient::new(client_settings, api_key)?;
        Ok(Self::new(api_client, glossary))
    }

    /// 加载系统提示词模板
    fn load_system_prompt(
        &self,
        source_lang: &str,
        target_lang: &str,
        source_text: &str,
    ) -> Result<String> {
        // 数据目录应按照以下顺序寻找，若不存在再寻找下一个：
        // 1. 当前目录下的提示词： ./data/
        // 2. 用户级数据目录下的提示词： ~/.local/share/pmt/data/
        let prompt_path = find_data_file_or_error("prompts/translate_system.txt")?;
        let mut prompt = fs::read_to_string(&prompt_path).map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::ValidationFailed(format!(
                "Failed to load prompt template from {}: {}",
                prompt_path.display(),
                e
            )))
        })?;

        // 提取源文本中的术语
        let mut all_found_terms = Vec::new();
        let found_terms = self.glossary.find_terms_in_text(source_text, source_lang);
        all_found_terms.extend(found_terms);

        // 去重
        all_found_terms.sort();
        all_found_terms.dedup();

        // 生成术语表CSV
        let glossary_csv = if all_found_terms.is_empty() {
            String::new()
        } else {
            // 合并所有术语表的术语
            let mut terms_count = 0;
            let mut csv_data = String::new();
            csv_data.push_str(&format!("{},{}", source_lang, target_lang));

            let source_terms: Vec<&str> = all_found_terms.iter().map(|s| s.as_str()).collect();

            let csv = self
                .glossary
                .to_csv(source_lang, target_lang, &source_terms);
            if !csv.is_empty() && csv.contains('\n') {
                // 跳过表头行（第一行）
                let lines: Vec<&str> = csv.lines().collect();
                if lines.len() > 1 {
                    for line in &lines[1..] {
                        if !line.trim().is_empty() {
                            csv_data.push('\n');
                            csv_data.push_str(line);
                            terms_count += 1;
                        }
                    }
                }
            }

            log::info!("Found {} terms for translation", terms_count);
            csv_data
        };

        // 替换模板中的占位符
        if !glossary_csv.is_empty() {
            prompt = prompt.replace("{{glossary_csv}}", &glossary_csv);
            log::debug!(
                "\n======DEBUG Using glossary CSV======\n{}\n======DEBUG END======\n",
                &glossary_csv
            );
        } else {
            prompt = prompt.replace("{{glossary_csv}}", "（无相关术语）");
        }

        Ok(prompt)
    }

    /// 翻译单个文本片段
    pub async fn translate_chunk(
        &self,
        chunk: &FileChunk,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<TranslationSlice> {
        // 加载系统提示词
        let source_text = &chunk.content;
        let system_prompt = self.load_system_prompt(source_lang, target_lang, source_text)?;

        // 准备消息
        let messages = vec![
            system_message(system_prompt),
            user_message(source_text.to_string()),
        ];

        let id = format!(
            "{}({}:{})",
            chunk.target_filename, chunk.start_line, chunk.end_line
        );
        log::info!(
            "Sending translation request [{}] with {} characters, estimated {} tokens...",
            id,
            source_text.chars().count(),
            estimate_mixed_tokens(&source_text)
        );
        // 调用API
        let response = self.api_client.chat_completions(messages).await?;

        log::info!(
            "Received translation response [{}], tokens used: {} + {} = {}",
            id,
            response.usage.prompt_tokens,
            response.usage.completion_tokens,
            response.usage.total_tokens
        );
        // 提取回复内容
        let translated_text = response
            .choices
            .first()
            .ok_or_else(|| {
                TranslationError::Translate(crate::error::TranslateError::InvalidResponse(
                    "No choices in API response".to_string(),
                ))
            })?
            .message
            .content
            .clone();

        // 验证格式
        let checked = self.validator.validate(&source_text, &translated_text);

        for problem in checked {
            log::warn!("Found issue in {}: {}", &chunk.target_filename, problem);
        }

        let slice = TranslationSlice {
            content: translated_text.to_owned(),
            start_line: chunk.start_line,
            end_line: chunk.end_line,
        };
        Ok(slice)
    }

    /// 批量翻译文本片段
    /// 每个片段独立翻译，适用于并发请求
    /// 返回按顺序排列的翻译结果
    /// 注意：此方法不会检查 chunks 的尺寸，请确保传入的 chunks 已经合理切分
    pub async fn translate_batch(
        &self,
        chunks: Vec<FileChunk>,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<TranslationSlice>> {
        let mut results: Vec<TranslationSlice> = Vec::new();
        let mut handles = Vec::new();
        for chunk in chunks {
            let chunk = chunk.to_owned();
            let handle = async move {
                self.translate_chunk(&chunk, &source_lang, &target_lang)
                    .await
            };
            handles.push(handle);
        }
        let translated = futures::future::join_all(handles).await;

        // 处理本批次的结果
        let mut has_error = false;
        let mut errors = String::new();
        for res in translated {
            match res {
                Ok(text) => results.push(text),
                Err(e) => {
                    errors.push_str(&format!("{} ", e));
                    has_error = true;
                }
            }
        }

        if has_error {
            return Err(TranslationError::AsyncError(errors.trim().to_string()));
        }

        Ok(results)
    }
}
