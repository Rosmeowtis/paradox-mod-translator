//! 翻译器模块
//!
//! 集成API客户端、术语表和提示词模板，执行翻译任务。

use crate::config::ClientSettings;
use crate::error::{Result, TranslationError};
use crate::translate::api::{ApiClient, system_message, user_message};
use crate::translate::glossary::Glossary;
use crate::translate::validator::FormatValidator;
use std::fs;

/// 翻译器
pub struct Translator {
    api_client: ApiClient,
    glossaries: Vec<Glossary>,
    validator: FormatValidator,
}

impl Translator {
    /// 创建新的翻译器
    pub fn new(api_client: ApiClient, glossaries: Vec<Glossary>) -> Self {
        Self {
            api_client,
            glossaries,
            validator: FormatValidator::new(),
        }
    }

    /// 从设置创建翻译器
    pub fn from_settings(
        client_settings: ClientSettings,
        glossaries: Vec<Glossary>,
    ) -> Result<Self> {
        let api_key = crate::config::load_openai_api_key()?;
        let api_client = ApiClient::new(client_settings, api_key)?;
        Ok(Self::new(api_client, glossaries))
    }

    /// 加载系统提示词模板
    fn load_system_prompt(
        &self,
        source_lang: &str,
        target_lang: &str,
        source_text: &str,
    ) -> Result<String> {
        let prompt_path = "data/prompts/translate_system.txt";
        let mut prompt = fs::read_to_string(prompt_path).map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::ValidationFailed(format!(
                "Failed to load prompt template: {}",
                e
            )))
        })?;

        // 提取源文本中的术语
        let mut all_found_terms = Vec::new();
        for glossary in &self.glossaries {
            let found_terms = glossary.find_terms_in_text(source_text, source_lang);
            all_found_terms.extend(found_terms);
        }

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
            for glossary in &self.glossaries {
                let csv = glossary.to_csv(source_lang, target_lang, &source_terms);
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
            }
            log::info!("Found {} terms for translation", terms_count);
            csv_data
        };

        // 替换模板中的占位符
        if !glossary_csv.is_empty() {
            prompt = prompt.replace("{{glossary_csv}}", &glossary_csv);
        } else {
            prompt = prompt.replace("{{glossary_csv}}", "（无相关术语）");
        }

        Ok(prompt)
    }

    /// 翻译单个文本片段
    pub async fn translate_text(
        &self,
        source_text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        // 加载系统提示词
        let system_prompt = self.load_system_prompt(source_lang, target_lang, source_text)?;

        // 准备消息
        let messages = vec![
            system_message(system_prompt),
            user_message(source_text.to_string()),
        ];

        // 调用API
        let response = self.api_client.chat_completions(messages).await?;

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
        self.validator.validate(source_text, &translated_text)?;

        Ok(translated_text)
    }

    /// 批量翻译文本片段
    pub async fn translate_batch(
        &self,
        source_texts: Vec<String>,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<Vec<String>> {
        let mut results = Vec::new();
        for text in source_texts {
            let translated = self.translate_text(&text, source_lang, target_lang).await?;
            results.push(translated);
        }
        Ok(results)
    }
}
