//! 验证器模块
//!
//! 验证翻译后的文本是否破坏了游戏特殊格式。

use crate::error::{Result, TranslationError};
use regex::Regex;

/// 特殊格式验证器
pub struct FormatValidator {
    /// £...£ 格式（图标）
    icon_pattern: Regex,
    /// $...$ 格式（变量）
    variable_pattern: Regex,
    /// §...§ 格式（颜色代码）
    color_pattern: Regex,
    // TODO 添加 [...] 格式（指令）
}

impl Default for FormatValidator {
    fn default() -> Self {
        Self {
            icon_pattern: Regex::new(r#"£[^£]+£"#).unwrap(),
            variable_pattern: Regex::new(r#"\$[^$]+\$"#).unwrap(),
            color_pattern: Regex::new(r#"§[^§]+§"#).unwrap(),
        }
    }
}

impl FormatValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self::default()
    }

    /// 验证翻译前后的格式是否一致
    pub fn validate(&self, original: &str, translated: &str) -> Result<()> {
        // 检查图标标记
        let original_icons: Vec<&str> = self
            .icon_pattern
            .find_iter(original)
            .map(|m| m.as_str())
            .collect();
        let translated_icons: Vec<&str> = self
            .icon_pattern
            .find_iter(translated)
            .map(|m| m.as_str())
            .collect();

        if original_icons != translated_icons {
            return Err(TranslationError::ValidationError(format!(
                "Icon markers mismatch. Original: {:?}, Translated: {:?}",
                original_icons, translated_icons
            )));
        }

        // 检查变量标记
        let original_vars: Vec<&str> = self
            .variable_pattern
            .find_iter(original)
            .map(|m| m.as_str())
            .collect();
        let translated_vars: Vec<&str> = self
            .variable_pattern
            .find_iter(translated)
            .map(|m| m.as_str())
            .collect();

        if original_vars != translated_vars {
            return Err(TranslationError::ValidationError(format!(
                "Variable markers mismatch. Original: {:?}, Translated: {:?}",
                original_vars, translated_vars
            )));
        }

        // 检查颜色标记
        let original_colors: Vec<&str> = self
            .color_pattern
            .find_iter(original)
            .map(|m| m.as_str())
            .collect();
        let translated_colors: Vec<&str> = self
            .color_pattern
            .find_iter(translated)
            .map(|m| m.as_str())
            .collect();

        if original_colors != translated_colors {
            return Err(TranslationError::ValidationError(format!(
                "Color markers mismatch. Original: {:?}, Translated: {:?}",
                original_colors, translated_colors
            )));
        }

        Ok(())
    }

    /// 提取所有特殊标记
    pub fn extract_markers(&self, text: &str) -> Vec<String> {
        let mut markers = Vec::new();
        markers.extend(
            self.icon_pattern
                .find_iter(text)
                .map(|m| m.as_str().to_string()),
        );
        markers.extend(
            self.variable_pattern
                .find_iter(text)
                .map(|m| m.as_str().to_string()),
        );
        markers.extend(
            self.color_pattern
                .find_iter(text)
                .map(|m| m.as_str().to_string()),
        );
        markers
    }
}
