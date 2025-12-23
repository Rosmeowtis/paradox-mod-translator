use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Preprocessing error: {0}")]
    Preprocess(#[from] PreprocessError),

    #[error("Translation error: {0}")]
    Translate(#[from] TranslateError),

    #[error("Postprocessing error: {0}")]
    Postprocess(#[from] PostprocessError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid YAML format: {0}")]
    InvalidYaml(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse TOML config: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

#[derive(Error, Debug)]
pub enum PreprocessError {
    #[error("YAML parsing error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("Invalid YAML structure: {0}")]
    InvalidStructure(String),

    #[error("Failed to fix YAML format: {0}")]
    FixFailed(String),

    #[error("File too large to process: {0}")]
    FileTooLarge(String),
}

#[derive(Error, Debug)]
pub enum TranslateError {
    #[error("API request failed: {0}")]
    ApiRequest(#[from] reqwest::Error),

    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    #[error("Glossary error: {0}")]
    GlossaryError(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Authentication failed")]
    AuthenticationFailed,
}

#[derive(Error, Debug)]
pub enum PostprocessError {
    #[error("Failed to merge translations: {0}")]
    MergeFailed(String),

    #[error("Failed to write output: {0}")]
    WriteFailed(String),

    #[error("Inconsistent translation slices")]
    InconsistentSlices,
}

pub type Result<T> = std::result::Result<T, TranslationError>;