//! 验证器模块
//!
//! 验证翻译后的文本是否破坏了游戏特殊格式。

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use regex::Regex;

/// 特殊格式验证器
pub struct FormatValidator {
    /// £...£ 格式（图标）
    icon_pattern: Regex,
    /// $...$ 格式（变量）
    variable_pattern: Regex,
    /// §x 格式（颜色代码），并非成对出现
    color_pattern: Regex,
    /// [...] 格式（指令）
    command_pattern: Regex,
}

impl Default for FormatValidator {
    fn default() -> Self {
        Self {
            icon_pattern: Regex::new(r#"£[^£]+£"#).unwrap(),
            variable_pattern: Regex::new(r#"\$[^$]+\$"#).unwrap(),
            color_pattern: Regex::new(r#"§[^§]"#).unwrap(),
            command_pattern: Regex::new(r#"\[[^\]]+\]"#).unwrap(),
        }
    }
}

#[derive(Debug)]
pub enum Problem {
    /// 键缺失
    MissingKey { key: String },
    /// 额外的键
    ExtraKey { key: String },
    /// 标记未找到
    PatternNotFound { key: String, original: String },
    /// 标记内容被改变
    PatternMismatch {
        key: String,
        original: String,
        translated: String,
    },
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem::MissingKey { key } => write!(f, "Missing key '{}'", key),
            Problem::ExtraKey { key } => write!(f, "Extra key '{}'", key),
            Problem::PatternNotFound { key, original } => {
                write!(f, "Pattern not found for key '{}' in '{}'", key, original)
            }
            Problem::PatternMismatch {
                key,
                original,
                translated,
            } => write!(
                f,
                "Pattern mismatch for key '{}': '{}' => '{}'",
                key, original, translated
            ),
        }
    }
}

impl FormatValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self::default()
    }

    /// 验证翻译前后的格式是否一致
    /// 传入的文本为一个切片的完整内容
    pub fn validate(&self, original: &str, translated: &str) -> Vec<Problem> {
        let mut problems = Vec::new();

        let original_items: Vec<(&str, &str)> = original
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ':');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    Some((key.trim(), value.trim()))
                } else {
                    None
                }
            })
            .collect();

        let translated_items: Vec<(&str, &str)> = translated
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ':');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    Some((key.trim(), value.trim()))
                } else {
                    None
                }
            })
            .collect();

        self.validate_keys(&original_items, &translated_items, &mut problems);
        let translated_items_map: HashMap<&str, &str> =
            translated_items.into_iter().map(|(k, v)| (k, v)).collect();
        for (key, original_value) in &original_items {
            if let Some(translated_value) = translated_items_map.get(key) {
                self.validate_patterns(key, original_value, translated_value, &mut problems);
            }
        }
        problems
    }

    /// 验证翻译前后条目的数量是否一致
    fn validate_keys(
        &self,
        original: &Vec<(&str, &str)>,
        translated: &Vec<(&str, &str)>,
        problems: &mut Vec<Problem>,
    ) -> usize {
        let mut problems_added = 0;
        let original_keys: HashSet<&str> = original.iter().map(|(k, _)| *k).collect();
        let translated_keys: HashSet<&str> = translated.iter().map(|(k, _)| *k).collect();
        let missing = original_keys.difference(&translated_keys);
        if missing.count() > 0 {
            for key in original_keys.difference(&translated_keys) {
                problems.push(Problem::MissingKey {
                    key: key.to_string(),
                });
                problems_added += 1;
            }
        }
        let extra = translated_keys.difference(&original_keys);
        if extra.count() > 0 {
            for key in translated_keys.difference(&original_keys) {
                problems.push(Problem::ExtraKey {
                    key: key.to_string(),
                });
                problems_added += 1;
            }
        }
        problems_added
    }

    /// 验证特定键的格式标记是否一致
    fn validate_patterns(
        &self,
        key: &str,
        original: &str,
        translated: &str,
        problems: &mut Vec<Problem>,
    ) -> usize {
        let mut problems_added = 0;
        for pattern in [
            &self.icon_pattern,
            &self.variable_pattern,
            &self.color_pattern,
            &self.command_pattern,
        ] {
            let mut original: Vec<&str> = pattern.find_iter(original).map(|m| m.as_str()).collect();
            let translated: Vec<&str> = pattern.find_iter(translated).map(|m| m.as_str()).collect();

            // 标记数量不匹配，检查缺失的标记，并将缺失标记从 original 中排除，以便后续检查内容一致性
            if original.len() != translated.len() {
                let original_set: HashSet<&str> = original.iter().cloned().collect();
                let translated_set: HashSet<&str> = translated.iter().cloned().collect();
                let missing: Vec<&&str> = original_set.difference(&translated_set).collect();
                for it in missing.iter() {
                    problems.push(Problem::PatternNotFound {
                        key: key.to_string(),
                        original: it.to_string(),
                    });
                    problems_added += 1;
                }

                // 排除缺失的标记
                original.retain(|it| !missing.contains(&it));
            }
            // 标记数量相等，检查内容是否一致
            for (it_original, it_translated) in original.iter().zip(translated.iter()) {
                if it_original != it_translated {
                    problems.push(Problem::PatternMismatch {
                        key: key.to_string(),
                        original: it_original.to_string(),
                        translated: it_translated.to_string(),
                    });
                    problems_added += 1;
                }
            }
        }
        problems_added
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
