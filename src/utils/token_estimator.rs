//! Token估算器
//!
//! 估算文本的token数量，用于文件切片。

/// 估算英文文本的token数量（近似）
pub fn estimate_english_tokens(text: &str) -> usize {
    // 简单估算：英文平均每个token约4个字符
    // 更准确的方法需要实际的分词器
    let char_count = text.chars().count();
    (char_count as f32 / 4.0).ceil() as usize
}

/// 估算中文文本的token数量（近似）
pub fn estimate_chinese_tokens(text: &str) -> usize {
    // 中文每个字符大约1-2个token
    // 使用保守估计：每个汉字1.5个token
    let chinese_char_count = text.chars()
        .filter(|c| is_cjk_character(*c))
        .count();
    let other_char_count = text.chars().count() - chinese_char_count;

    // 中文字符：1.5 token/字符，其他字符：0.25 token/字符（英文比例）
    ((chinese_char_count as f32 * 1.5) + (other_char_count as f32 * 0.25)).ceil() as usize
}

/// 估算混合文本的token数量
pub fn estimate_mixed_tokens(text: &str) -> usize {
    // 简单实现：检查是否包含中文字符
    let has_chinese = text.chars().any(is_cjk_character);

    if has_chinese {
        estimate_chinese_tokens(text)
    } else {
        estimate_english_tokens(text)
    }
}

/// 检查字符是否为CJK（中日韩）字符
pub fn is_cjk_character(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |  // CJK统一表意文字
        '\u{3400}'..='\u{4DBF}' |  // CJK统一表意文字扩展A
        '\u{20000}'..='\u{2A6DF}' | // CJK统一表意文字扩展B
        '\u{2A700}'..='\u{2B73F}' | // CJK统一表意文字扩展C
        '\u{2B740}'..='\u{2B81F}' | // CJK统一表意文字扩展D
        '\u{2B820}'..='\u{2CEAF}' | // CJK统一表意文字扩展E
        '\u{F900}'..='\u{FAFF}' |  // CJK兼容表意文字
        '\u{2F800}'..='\u{2FA1F}'   // CJK兼容表意文字补充
    )
}

/// 根据token限制计算最大字符数
pub fn max_chars_for_tokens(max_tokens: usize, is_chinese: bool) -> usize {
    if is_chinese {
        // 中文：每个token约0.67个字符
        (max_tokens as f32 * 0.67).ceil() as usize
    } else {
        // 英文：每个token约4个字符
        max_tokens * 4
    }
}