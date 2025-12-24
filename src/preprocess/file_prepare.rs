/// 生成目标文件名（例如将 l_english 替换为目标语言），并确保文件后缀名为 .yml
pub fn generate_target_filename(
    source_filename: &str,
    source_lang: &str,
    target_lang: &str,
) -> String {
    source_filename
        .replace(&format!("l_{}", source_lang), &format!("l_{}", target_lang))
        .replace(".yaml", ".yml") // 统一使用 .yml 扩展名
}
