//! Paradox Mod Translator - AI-powered translation tool for Paradox game mods.

pub mod config;
pub mod postprocess;
pub mod preprocess;
pub mod translate;
pub mod utils;

pub mod error;

// Re-export commonly used types
pub use error::{Result, TranslationError};

use crate::{
    preprocess::{fix_yaml_content, trim_lang_header},
    translate::FormatValidator,
};

/// 执行翻译任务
pub async fn translate_task(
    task: config::TranslationTask,
    client_settings: config::ClientSettings,
) -> Result<()> {
    use crate::translate::{Translator, load_glossaries_from_task};
    use std::fs;
    use walkdir::WalkDir;

    log::info!("Starting translation task");
    log::info!("Source language: {}", task.source_lang);
    log::info!("Target languages: {:?}", task.target_langs);

    // 1. 加载术语表
    let merged_glossary = load_glossaries_from_task(&task)?;

    // 2. 创建翻译器
    let max_chunk_size = client_settings.max_chunk_size;
    let translator = Translator::from_settings(client_settings, merged_glossary)?;

    // 3. 遍历源目录中的文件
    let source_dir = task.source_dir();
    log::info!("Reading source files from: {:?}", source_dir);

    let mut source_files = Vec::new();
    for entry in WalkDir::new(&source_dir) {
        let entry = entry.map_err(|e| {
            TranslationError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WalkDir error: {}", e),
            ))
        })?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    source_files.push(path.to_path_buf());
                }
            }
        }
    }

    log::info!("Found {} source files", source_files.len());

    // 4. 对每个目标语言进行翻译
    for target_lang in &task.target_langs {
        log::info!("Translating to: {}", target_lang);

        let target_dir = task.target_dir(target_lang);
        log::info!("Output directory: {:?}", target_dir);

        // 创建目标目录
        fs::create_dir_all(&target_dir)?;

        for source_file in &source_files {
            log::info!("Processing file: {:?}", source_file);
            translate_one_file(
                &translator,
                &task.source_lang,
                target_lang,
                max_chunk_size,
                &target_dir,
                source_file,
            )
            .await?;
        }
    }

    log::info!("Translation task completed successfully!");
    Ok(())
}

pub async fn translate_one_file(
    translator: &translate::Translator,
    source_lang: &str,
    target_lang: &str,
    max_chunk_size: usize,
    target_dir: &std::path::PathBuf,
    source_file: &std::path::PathBuf,
) -> Result<()> {
    use crate::postprocess::{TranslationSlice, reconstruct_yaml_file, write_translated_file};
    use crate::preprocess::{fix_yaml_content, generate_target_filename, trim_lang_header};
    use crate::translate::split_yaml_content;
    use std::fs;

    // 算出输出文件路径
    let filename = source_file
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| TranslationError::FileNotFound("Invalid filename".to_string()))?;
    let target_filename = generate_target_filename(filename, &source_lang, target_lang);
    let output_path = target_dir.join(&target_filename);

    // 读取源文件内容
    let content = fs::read_to_string(source_file)?;
    // 去除 BOM 头
    let content = if content.starts_with("\u{FEFF}") {
        content.trim_start_matches("\u{FEFF}")
    } else {
        &content
    }
    .to_string();
    // 去除语言头标记
    let (_original_header, content) = trim_lang_header(&source_lang, &content);
    // 修复YAML文件中的格式问题
    let content = fix_yaml_content(&content)?;
    // 切片
    let chunks = split_yaml_content(&target_filename, &content, max_chunk_size)?;
    log::info!("File split into {} chunks", chunks.len());

    // 翻译每个切片
    let mut translated_chunks = Vec::new();
    for chunk in chunks {
        log::trace!(
            "\n======TRACE Translating chunk======\n{}\n======TRACE END======\n",
            &chunk.content
        );

        let translated_content = translator
            .translate_chunk(&chunk, &source_lang, target_lang)
            .await?;

        log::trace!(
            "\n======TRACE Translated======\n{}\n======TRACE END======\n",
            &translated_content
        );

        translated_chunks.push(TranslationSlice {
            content: translated_content,
            start_line: chunk.start_line,
            end_line: chunk.end_line,
        });
    }
    let reconstructed = reconstruct_yaml_file(translated_chunks, &target_lang)?;

    write_translated_file(&reconstructed, &output_path, true)?;
    log::info!("Successfully translated: {:?}", output_path);
    Ok(())
}

pub async fn validate_translation(task: config::TranslationTask) -> Result<()> {
    use walkdir::WalkDir;

    log::info!("Starting translation validation");
    log::info!("Source language: {}", task.source_lang);
    log::info!("Target languages: {:?}", task.target_langs);

    let source_dir = task.source_dir();
    log::info!("Reading source files from: {:?}", source_dir);

    let mut source_files = Vec::new();
    for entry in WalkDir::new(&source_dir) {
        let entry = entry.map_err(|e| {
            TranslationError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WalkDir error: {}", e),
            ))
        })?;
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "yml" || ext == "yaml" {
                    source_files.push(path.to_path_buf());
                }
            }
        }
    }

    log::info!("Found {} source files", source_files.len());

    for target_lang in &task.target_langs {
        log::info!(
            "Validating translations for target language: {}",
            target_lang
        );

        let target_dir = task.target_dir(target_lang);
        log::info!("Looking for translated files in: {:?}", target_dir);

        for source_file in &source_files {
            let filename = source_file
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| TranslationError::FileNotFound("Invalid filename".to_string()))?;
            let target_filename =
                preprocess::generate_target_filename(filename, &task.source_lang, target_lang);
            let output_path = target_dir.join(&target_filename);

            if output_path.exists() {
                validate_one_file(&task.source_lang, target_lang, source_file, &output_path)
                    .await?;
            } else {
                log::warn!("Missing translated file: {:?}", output_path);
            }
        }
    }

    log::info!("Translation validation completed");
    Ok(())
}

pub async fn validate_one_file(
    source_lang: &str,
    target_lang: &str,
    source_file: &std::path::PathBuf,
    translated_file: &std::path::PathBuf,
) -> Result<()> {
    use std::fs;

    let source = fs::read_to_string(source_file)?;
    let translated = fs::read_to_string(translated_file)?;

    // 去除 BOM 头
    let source = source.trim_start_matches("\u{FEFF}");
    let translated = translated.trim_start_matches("\u{FEFF}");

    // 去除语言头标记
    let (_, source) = trim_lang_header(source_lang, source);
    let (_, translated) = trim_lang_header(target_lang, translated);

    // 修复YAML文件中的格式问题
    let source = fix_yaml_content(&source)?;
    let translated = fix_yaml_content(&translated)?;

    let validator = FormatValidator::new();
    // 检查 key 的数量和名称是否一一对应
    let issues = validator.validate(&source, &translated);
    if issues.is_empty() {
        log::info!(
            "[x] Validation passed for file {}",
            translated_file.display()
        );
        return Ok(());
    }
    log::warn!("[ ] Issues in {}:", translated_file.display());
    for (i, issue) in issues.iter().enumerate() {
        log::warn!("  {}. {}", i + 1, issue);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Keep existing test structure for now
    #[test]
    fn it_works() {
        // Simple placeholder test
        assert_eq!(2 + 2, 4);
    }
}
