pub fn detect_language(text: &str) -> &'static str {
    let chinese_chars = text
        .chars()
        .filter(|c| {
            (*c as u32) >= 0x4e00 && (*c as u32) <= 0x9fff // 基本汉字范围
        })
        .count();

    let total_chars = text.chars().filter(|c| !c.is_whitespace()).count();

    if total_chars == 0 {
        return "unknown";
    }

    let chinese_ratio = chinese_chars as f64 / total_chars as f64;
    if chinese_ratio > 0.3 {
        // 如果超过30%的字符是中文，则认为是中文
        "zh-CN"
    } else {
        "en"
    }
}
