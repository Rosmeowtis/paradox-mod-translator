//! Paradox Mod Translator 主程序
//!
//! 命令行接口和主工作流程。

use clap::{Parser, Subcommand};
use ftail::Ftail;
use log::{LevelFilter, Log};
use paradox_mod_translator::config::{TranslationTask, load_openai_api_key};
use paradox_mod_translator::error::{Result, TranslationError};
use paradox_mod_translator::{translate_task, validate_translation};
use std::path::{Path, PathBuf};

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
    /// 在已经完成翻译的情况下，跳过翻译任务，只检查翻译结果是否符合要求
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
    use paradox_mod_translator::utils::ColorfulConsoleLogger;
    Ftail::new()
        .single_file(
            Path::new("paradox-mod-translator.log"),
            true,
            LevelFilter::Trace,
        )
        // 简约控制台输出
        .custom(
            |config| Box::new(ColorfulConsoleLogger { config }) as Box<dyn Log + Send + Sync>,
            LevelFilter::Info,
        )
        .init()
        .unwrap();

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
            let (client_settings, tasks) = TranslationTask::from_file(&task_file)?;
            log::info!(
                "Configuration loaded successfully, found {} task(s)",
                tasks.len()
            );

            for (i, task) in tasks.iter().enumerate() {
                log::info!("Processing task {}/{}", i + 1, tasks.len());
                log::debug!("Source language: {}", task.source_lang);
                log::debug!("Target languages: {:?}", task.target_langs);
                log::debug!("Glossaries: {:?}", task.glossaries);

                // 执行翻译任务
                translate_task(task.clone(), client_settings.clone()).await?;
            }

            log::info!("All translation tasks completed!");
            Ok(())
        }
        Commands::Validate { task_file } => {
            log::info!("Validating translated task: {:?}", task_file);

            let (_client_settings, tasks) = TranslationTask::from_file(&task_file)?;

            log::info!("Configuration is loaded! Found {} task(s)", tasks.len());

            for (i, task) in tasks.iter().enumerate() {
                log::info!("Task {}:", i + 1);
                log::info!("  - Source language: {}", task.source_lang);
                log::info!("  - Target languages: {}", task.target_langs.join(", "));
                log::info!("  - Glossaries: {}", task.glossaries.join(", "));
                log::info!("  - Localisation directory: {:?}", task.localisation_dir);
            }

            for task in tasks {
                validate_translation(task).await?;
            }

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
