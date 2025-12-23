//! Paradox Mod Translator 主程序
//!
//! 命令行接口和主工作流程。

use clap::{Parser, Subcommand};
use paradox_mod_translator::config::{TranslationTask, load_openai_api_key};
use paradox_mod_translator::error::{Result, TranslationError};
use paradox_mod_translator::translate_task;
use pretty_env_logger;
use std::path::PathBuf;

/// 命令行参数
#[derive(Parser)]
#[command(name = "pmt")]
#[command(about = "Paradox Mod Translator - AI-powered translation tool for Paradox game mods")]
#[command(version, author, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// 子命令
#[derive(Subcommand)]
enum Commands {
    /// 执行翻译任务
    Translate {
        /// 任务配置文件路径
        #[arg(value_name = "TASK_FILE")]
        task_file: PathBuf,

        /// 启用详细输出
        #[arg(short, long)]
        verbose: bool,

        /// 跳过预处理（直接使用已修复的文件）
        #[arg(long)]
        skip_preprocess: bool,

        /// 跳过验证（不检查格式标记）
        #[arg(long)]
        skip_validation: bool,
    },
    /// 验证配置文件
    Validate {
        /// 任务配置文件路径
        #[arg(value_name = "TASK_FILE")]
        task_file: PathBuf,
    },
    /// 检查API密钥
    CheckApi,
}

/// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    pretty_env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Translate {
            task_file,
            verbose,
            skip_preprocess: _skip_preprocess,
            skip_validation: _skip_validation,
        } => {
            if verbose {
                log::info!("Starting translation task from: {:?}", task_file);
            }

            // 检查API密钥
            if !paradox_mod_translator::config::has_api_key() {
                log::error!("OPENAI_API_KEY environment variable is not set");
                log::info!("Please set OPENAI_API_KEY environment variable or create a .env file");
                return Err(TranslationError::MissingEnvVar(
                    "OPENAI_API_KEY environment variable is required".to_string(),
                ));
            }

            // 加载配置
            log::info!("Loading task configuration...");
            let (client_settings, task) = TranslationTask::from_file(&task_file)?;
            log::info!("Configuration loaded successfully");
            log::debug!("Source language: {}", task.source_lang);
            log::debug!("Target languages: {:?}", task.target_langs);
            log::debug!("Glossaries: {:?}", task.glossaries);

            // 执行翻译任务
            translate_task(task, client_settings).await?;

            log::info!("Translation task completed!");
            Ok(())
        }
        Commands::Validate { task_file } => {
            log::info!("Validating configuration file: {:?}", task_file);

            let (client_settings, task) = TranslationTask::from_file(&task_file)?;

            log::info!("Configuration is valid!");
            log::info!("- Source language: {}", task.source_lang);
            log::info!("- Target languages: {}", task.target_langs.join(", "));
            log::info!("- Glossaries: {}", task.glossaries.join(", "));
            log::info!("- Localisation directory: {:?}", task.localisation_dir);
            log::info!("- Client settings:");
            log::info!("  * API base: {}", client_settings.api_base);
            log::info!("  * Model: {}", client_settings.model);
            log::info!("  * Temperature: {}", client_settings.temperature);

            Ok(())
        }
        Commands::CheckApi => {
            if paradox_mod_translator::config::has_api_key() {
                log::info!("API key is configured");
                match load_openai_api_key() {
                    Ok(key) => {
                        let masked_key = if key.len() > 8 {
                            format!("{}...{}", &key[0..4], &key[key.len() - 4..])
                        } else {
                            "***".to_string()
                        };
                        log::info!("API key (masked): {}", masked_key);
                    }
                    Err(e) => {
                        log::error!("Failed to load API key: {}", e);
                        return Err(e);
                    }
                }
            } else {
                log::error!("API key is not configured");
                log::info!("Please set OPENAI_API_KEY environment variable");
                return Err(TranslationError::MissingEnvVar(
                    "OPENAI_API_KEY environment variable is required".to_string(),
                ));
            }

            Ok(())
        }
    }
}
