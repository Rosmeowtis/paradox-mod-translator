//! Paradox Mod Translator - AI-powered translation tool for Paradox game mods.

pub mod config;
pub mod postprocess;
pub mod preprocess;
pub mod translate;
pub mod utils;

pub mod error;

// Re-export commonly used types
pub use error::{Result, TranslationError};

/// 执行翻译任务
pub async fn translate_task(
    task: config::TranslationTask,
    client_settings: config::ClientSettings,
) -> Result<()> {
    use postprocess::{TranslationSlice, reconstruct_yaml_file, write_translated_file};
    use preprocess::{fix_yaml_content, generate_target_filename, trim_lang_header};
    use std::fs;
    use translate::{Glossary, Translator, split_yaml_content};
    use walkdir::WalkDir;

    log::info!("Starting translation task");
    log::info!("Source language: {}", task.source_lang);
    log::info!("Target languages: {:?}", task.target_langs);

    // 1. 加载术语表
    let mut glossaries = Vec::new();
    for glossary_name in &task.glossaries {
        // 首先尝试用户自定义术语表
        let custom_path = format!("data/glossary_custom/{}.json", glossary_name);
        let path = if std::path::Path::new(&custom_path).exists() {
            custom_path
        } else {
            format!("data/glossary/{}.json", glossary_name)
        };

        log::debug!("Loading glossary: {}", path);
        let glossary = Glossary::from_json_file(&path)?;
        let glossary_len = glossary.len();
        glossaries.push(glossary);
        log::info!(
            "Loaded glossary '{}' with {} entries",
            glossary_name,
            glossary_len
        );
    }
    // 合并多个术语表到同一个Glossary对象中
    let merged_glossary = Glossary::merge_glossaries(&glossaries);

    // 2. 创建翻译器
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

            // 读取源文件，忽略BOM头
            let content = fs::read_to_string(source_file)?;
            let content = if content.starts_with("\u{FEFF}") {
                content.trim_start_matches("\u{FEFF}")
            } else {
                &content
            }
            .to_string();

            // 提取文件名
            let filename = source_file
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| TranslationError::FileNotFound("Invalid filename".to_string()))?;

            // 生成目标文件名
            let target_filename =
                generate_target_filename(filename, &task.source_lang, target_lang);
            let output_path = target_dir.join(&target_filename);

            // 移除语言头（如果存在）
            let (_original_header, content) = trim_lang_header(&task, content);

            // 预处理：修复YAML
            let content = fix_yaml_content(&content)?;

            // 切片（假设最大token数为2000，实际应根据模型调整）
            let max_chunk_size = 2000;
            let chunks = split_yaml_content(&target_filename, &content, max_chunk_size)?;

            log::info!("File split into {} chunks", chunks.len());

            // 翻译每个切片
            let mut translated_chunks = Vec::new();
            for chunk in chunks {
                log::debug!(
                    "\n======DEBUG Translating chunk======\n{}\n======DEBUG END======\n",
                    &chunk.content
                );

                let translated_content = translator
                    .translate_chunk(&chunk, &task.source_lang, target_lang)
                    .await?;

                log::debug!(
                    "\n======DEBUG Translated======\n{}\n======DEBUG END======\n",
                    &translated_content
                );

                translated_chunks.push(TranslationSlice {
                    content: translated_content,
                    start_line: chunk.start_line,
                    end_line: chunk.end_line,
                });
            }

            // 后处理：合并切片并重建YAML文件
            let reconstructed = reconstruct_yaml_file(translated_chunks, &target_lang)?;

            // 写入目标文件
            write_translated_file(&reconstructed, &output_path, true)?;
            log::info!("Successfully translated: {:?}", output_path);
        }
    }

    log::info!("Translation task completed successfully!");
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
