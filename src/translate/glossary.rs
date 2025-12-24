//! 术语表模块
//!
//! 加载和管理翻译术语表。每个术语表提供多语言对照。

use crate::error::{Result, TranslationError};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// 多语言术语条目
///
/// 字段按语言使用量排序，使用数字键名进行序列化/反序列化以节省空间：
/// 1: english, 2: simp_chinese, 3: spanish, 4: french, 5: braz_por,
/// 6: russian, 7: german, 8: japanese, 9: korean, 10: polish
/// 反序列化属性写在后文的 RawItem 结构体中。
#[derive(Debug, Clone, Serialize)]
pub struct GlossaryItem {
    pub english: Option<String>,      // 1
    pub simp_chinese: Option<String>, // 2
    pub spanish: Option<String>,      // 3
    pub french: Option<String>,       // 4
    pub braz_por: Option<String>,     // 5
    pub russian: Option<String>,      // 6
    pub german: Option<String>,       // 7
    pub japanese: Option<String>,     // 8
    pub korean: Option<String>,       // 9
    pub polish: Option<String>,       // 10
}

impl<'de> Deserialize<'de> for GlossaryItem {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawItem {
            #[serde(rename = "1", default)]
            english: Option<String>,
            #[serde(rename = "2", default)]
            simp_chinese: Option<String>,
            #[serde(rename = "3", default)]
            spanish: Option<String>,
            #[serde(rename = "4", default)]
            french: Option<String>,
            #[serde(rename = "5", default)]
            braz_por: Option<String>,
            #[serde(rename = "6", default)]
            russian: Option<String>,
            #[serde(rename = "7", default)]
            german: Option<String>,
            #[serde(rename = "8", default)]
            japanese: Option<String>,
            #[serde(rename = "9", default)]
            korean: Option<String>,
            #[serde(rename = "10", default)]
            polish: Option<String>,
        }

        let raw = RawItem::deserialize(deserializer)?;

        // 检查是否至少有一个字段有值
        let has_value = raw.english.is_some()
            || raw.simp_chinese.is_some()
            || raw.spanish.is_some()
            || raw.french.is_some()
            || raw.braz_por.is_some()
            || raw.russian.is_some()
            || raw.german.is_some()
            || raw.japanese.is_some()
            || raw.korean.is_some()
            || raw.polish.is_some();

        if !has_value {
            return Err(<D as serde::Deserializer<'de>>::Error::custom(
                "GlossaryItem must contain at least one language field",
            ));
        }

        Ok(GlossaryItem {
            english: raw.english,
            simp_chinese: raw.simp_chinese,
            spanish: raw.spanish,
            french: raw.french,
            braz_por: raw.braz_por,
            russian: raw.russian,
            german: raw.german,
            japanese: raw.japanese,
            korean: raw.korean,
            polish: raw.polish,
        })
    }
}

impl GlossaryItem {
    /// 获取指定语言的术语
    pub fn get(&self, lang: &str) -> Option<&str> {
        match lang {
            "english" => self.english.as_deref(),
            "simp_chinese" => self.simp_chinese.as_deref(),
            "spanish" => self.spanish.as_deref(),
            "french" => self.french.as_deref(),
            "braz_por" => self.braz_por.as_deref(),
            "russian" => self.russian.as_deref(),
            "german" => self.german.as_deref(),
            "japanese" => self.japanese.as_deref(),
            "korean" => self.korean.as_deref(),
            "polish" => self.polish.as_deref(),
            _ => None,
        }
    }

    /// 检查是否包含指定语言的术语
    pub fn has_language(&self, lang: &str) -> bool {
        self.get(lang).is_some()
    }

    /// 获取所有有值的语言和术语
    pub fn all_terms(&self) -> Vec<(&'static str, &str)> {
        let mut terms = Vec::new();
        if let Some(term) = self.english.as_deref() {
            terms.push(("english", term));
        }
        if let Some(term) = self.simp_chinese.as_deref() {
            terms.push(("simp_chinese", term));
        }
        if let Some(term) = self.spanish.as_deref() {
            terms.push(("spanish", term));
        }
        if let Some(term) = self.french.as_deref() {
            terms.push(("french", term));
        }
        if let Some(term) = self.braz_por.as_deref() {
            terms.push(("braz_por", term));
        }
        if let Some(term) = self.russian.as_deref() {
            terms.push(("russian", term));
        }
        if let Some(term) = self.german.as_deref() {
            terms.push(("german", term));
        }
        if let Some(term) = self.japanese.as_deref() {
            terms.push(("japanese", term));
        }
        if let Some(term) = self.korean.as_deref() {
            terms.push(("korean", term));
        }
        if let Some(term) = self.polish.as_deref() {
            terms.push(("polish", term));
        }
        terms
    }
}

/// 术语表
#[derive(Debug, Clone, Default)]
pub struct Glossary {
    /// 术语索引：key -> GlossaryItem
    entries: HashMap<String, GlossaryItem>,
}

impl Glossary {
    /// 从JSON文件加载术语表
    ///
    /// 术语表文件格式为多语言术语表：
    /// ```json
    /// {
    ///   "energy": {"1": "energy", "2": "能量", "3": "energía"},
    ///   "minerals": {"1": "minerals", "2": "矿物"}
    /// }
    /// ```
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::GlossaryError(e.to_string()))
        })?;

        let raw: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::GlossaryError(e.to_string()))
        })?;

        let mut entries = HashMap::new();

        match raw {
            serde_json::Value::Object(obj) => {
                for (key, value) in obj {
                    match serde_json::from_value::<GlossaryItem>(value.clone()) {
                        Ok(glossary_item) => {
                            entries.insert(key, glossary_item);
                        }
                        Err(e) => {
                            // 无法解析的值，记录警告并跳过
                            log::warn!("无法解析术语表条目: key={}, error={}", key, e);
                        }
                    }
                }
            }
            _ => {
                return Err(TranslationError::Translate(
                    crate::error::TranslateError::GlossaryError(
                        "术语表文件必须是JSON对象".to_string(),
                    ),
                ));
            }
        }

        Ok(Self { entries })
    }

    /// 获取源语言到目标语言的翻译映射
    ///
    /// 返回HashMap<源术语, 目标术语>，仅包含同时具有源语言和目标语言的条目
    pub fn get_translation_map(
        &self,
        source_lang: &str,
        target_lang: &str,
    ) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (_key, item) in &self.entries {
            if let Some(source_term) = item.get(source_lang) {
                if let Some(target_term) = item.get(target_lang) {
                    map.insert(source_term.to_string(), target_term.to_string());
                }
            }
        }
        map
    }

    /// 应用术语表到文本（从源语言翻译到目标语言）
    pub fn apply(&self, text: &str, source_lang: &str, target_lang: &str) -> String {
        let translation_map = self.get_translation_map(source_lang, target_lang);
        let mut result = text.to_string();
        for (source, target) in translation_map {
            // 简单的替换，需要改进为单词边界匹配
            result = result.replace(&source, &target);
        }
        result
    }

    /// 获取术语表大小
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取所有条目
    pub fn entries(&self) -> &HashMap<String, GlossaryItem> {
        &self.entries
    }

    /// 将一组术语格式化为CSV，以便嵌入 prompt 中
    /// 输出格式为
    ///
    /// ```csv
    /// <source_lang>,<target_lang>
    /// <source_term1>,<target_term1>
    /// <source_term2>,<target_term2>
    /// ```
    ///
    /// 例如
    ///
    /// ```csv
    /// english,simp_chinese
    /// energy,能量
    /// minerals,矿物
    /// ```
    pub fn to_csv(&self, source_lang: &str, target_lang: &str, source_words: &[&str]) -> String {
        let mut wtr = String::with_capacity(1024);
        // header
        wtr.push_str(&format!("{},{}\n", source_lang, target_lang));

        for word in source_words {
            if let Some(item) = self.entries.get(*word) {
                if let Some(source_term) = item.get(source_lang) {
                    if let Some(target_term) = item.get(target_lang) {
                        wtr.push_str(&format!("{},{}\n", source_term, target_term));
                    }
                }
            }
        }

        wtr
    }

    /// 发现待翻译文本中存在的术语表条目
    pub fn find_terms_in_text(&self, text: &str, source_lang: &str) -> Vec<String> {
        let mut found_terms = Vec::new();
        let text = text.to_lowercase();
        for (_key, item) in &self.entries {
            if let Some(source_term) = item.get(source_lang) {
                if text.contains(source_term) {
                    found_terms.push(source_term.to_string());
                }
            }
        }
        found_terms
    }

    /// 合并多个术语表到一个术语表
    pub fn merge_glossaries(glossaries: &[Glossary]) -> Glossary {
        let mut merged_entries = HashMap::new();
        for glossary in glossaries {
            for (key, item) in &glossary.entries {
                merged_entries.insert(key.clone(), item.clone());
            }
        }
        Glossary {
            entries: merged_entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glossary_item_deserialize_new_format() {
        let json = r#"{"1": "energy", "2": "能量", "3": "energía"}"#;
        let item: GlossaryItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.english, Some("energy".to_string()));
        assert_eq!(item.simp_chinese, Some("能量".to_string()));
        assert_eq!(item.spanish, Some("energía".to_string()));
        assert!(item.french.is_none());
    }

    #[test]
    fn test_glossary_item_deserialize_empty_fails() {
        let json = r#"{}"#;
        let result: std::result::Result<GlossaryItem, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_glossary_item_get() {
        let item = GlossaryItem {
            english: Some("energy".to_string()),
            simp_chinese: Some("能量".to_string()),
            spanish: None,
            french: None,
            braz_por: None,
            russian: None,
            german: None,
            japanese: None,
            korean: None,
            polish: None,
        };
        assert_eq!(item.get("english"), Some("energy"));
        assert_eq!(item.get("simp_chinese"), Some("能量"));
        assert_eq!(item.get("spanish"), None);
        assert_eq!(item.get("invalid"), None);
    }

    #[test]
    fn test_glossary_load() {
        let json = r#"{
            "energy": {"1": "energy", "2": "能量", "3": "energía"},
            "minerals": {"1": "minerals", "2": "矿物"}
        }"#;
        let glossary = from_json_file_content(json).unwrap();
        assert_eq!(glossary.len(), 2);

        let entries = glossary.entries();
        let energy_item = entries.get("energy").unwrap();
        assert_eq!(energy_item.english, Some("energy".to_string()));
        assert_eq!(energy_item.simp_chinese, Some("能量".to_string()));
        assert_eq!(energy_item.spanish, Some("energía".to_string()));
    }

    #[test]
    fn test_glossary_translation_map() {
        let json = r#"{
            "energy": {"1": "energy", "2": "能量"},
            "minerals": {"1": "minerals", "2": "矿物"}
        }"#;
        let glossary = from_json_file_content(json).unwrap();
        let map = glossary.get_translation_map("english", "simp_chinese");
        assert_eq!(map.get("energy"), Some(&"能量".to_string()));
        assert_eq!(map.get("minerals"), Some(&"矿物".to_string()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_glossary_apply() {
        let json = r#"{
            "energy": {"1": "energy", "2": "能量"},
            "minerals": {"1": "minerals", "2": "矿物"}
        }"#;
        let glossary = from_json_file_content(json).unwrap();
        let text = "We need more energy and minerals.";
        let translated = glossary.apply(text, "english", "simp_chinese");
        assert_eq!(translated, "We need more 能量 and 矿物.");
    }

    /// 辅助函数：从字符串内容加载术语表（用于测试）
    fn from_json_file_content(content: &str) -> Result<Glossary> {
        let raw: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            TranslationError::Translate(crate::error::TranslateError::GlossaryError(e.to_string()))
        })?;

        let mut entries = HashMap::new();
        if let serde_json::Value::Object(obj) = raw {
            for (key, value) in obj {
                match serde_json::from_value::<GlossaryItem>(value.clone()) {
                    Ok(glossary_item) => {
                        entries.insert(key, glossary_item);
                    }
                    Err(e) => {
                        // 无法解析的值，记录警告并跳过
                        log::warn!("无法解析术语表条目: key={}, error={}", key, e);
                    }
                }
            }
        } else {
            return Err(TranslationError::Translate(
                crate::error::TranslateError::GlossaryError("术语表文件必须是JSON对象".to_string()),
            ));
        }

        Ok(Glossary { entries })
    }
}
