use tauri::LogicalSize;

use crate::my_windows;


// Font configuration constants
const FONT_SIZE: f64 = 14.0; // Actual font size 14px

// Adjusted character width coefficients
const CJK_WIDTH_RATIO: f64 = 1.0; // CJK monospace characters
const ASCII_WIDE_RATIO: f64 = 0.7; // Wide letters (W, M, w, m) - increased to 0.7
const ASCII_NORMAL_RATIO: f64 = 0.55; // Normal ASCII characters - increased to 0.55
const ASCII_NARROW_RATIO: f64 = 0.35; // Narrow characters - increased to 0.35
const SPACE_RATIO: f64 = 0.4; // **Define space width separately**
const TAB_WIDTH_RATIO: f64 = 2.0;

pub fn calculate_text_width(content: &str) -> LogicalSize<f64> {
    let mut total_width: f64 = 0.0;

    for c in content.chars() {
        let char_width = match c {
            // Chinese characters (CJK unified ideographs)
            '\u{4E00}'..='\u{9FFF}' |  // Basic Han characters
            '\u{3400}'..='\u{4DBF}' |  // Extension A
            '\u{20000}'..='\u{2A6DF}' | // Extension B
            '\u{2A700}'..='\u{2B73F}' | // Extension C
            '\u{2B740}'..='\u{2B81F}' | // Extension D
            '\u{2B820}'..='\u{2CEAF}' | // Extension E
            '\u{F900}'..='\u{FAFF}' |   // Compatibility Han characters
            '\u{2F800}'..='\u{2FA1F}'   // Compatibility supplement
            => FONT_SIZE * CJK_WIDTH_RATIO,
            // Full-width characters (including full-width punctuation, Japanese kana, etc.)
            '\u{FF01}'..='\u{FF5E}' |  // Full-width ASCII
            '\u{3000}'..='\u{303F}' |  // CJK punctuation
            '\u{3040}'..='\u{309F}' |  // Hiragana
            '\u{30A0}'..='\u{30FF}'    // Katakana
            => FONT_SIZE * CJK_WIDTH_RATIO,
            // Korean characters
            '\u{AC00}'..='\u{D7AF}' |  // Korean syllables
            '\u{1100}'..='\u{11FF}'    // Korean letters
            => FONT_SIZE * CJK_WIDTH_RATIO,
            // Emojis and special symbols
            '\u{1F300}'..='\u{1F9FF}' | // Emojis
            '\u{2600}'..='\u{26FF}' |   // Miscellaneous symbols
            '\u{2700}'..='\u{27BF}'     // Decorative symbols
            => FONT_SIZE * CJK_WIDTH_RATIO,
            // Tab character
            '\t' => FONT_SIZE * TAB_WIDTH_RATIO,
            // Line feed (treated as normal space)
            '\n' | '\r' => FONT_SIZE * ASCII_NORMAL_RATIO,
            // Modified in matching logic:
            _ if c.is_ascii() => {
                match c {
                    // Handle space separately
                    ' ' => FONT_SIZE * SPACE_RATIO,
                    // Wide letters
                    'W' | 'M' | 'w' | 'm' | '@' | '%' | '#' | '&' | '$'
                    => FONT_SIZE * ASCII_WIDE_RATIO,
                    // Narrow characters (excluding space)
                    'i' | 'l' | 'I' | 'j' | 't' | 'f' | 'r' |
                    '.' | ',' | ':' | ';' | '\'' | '!' | '|' | '`'
                    => FONT_SIZE * ASCII_NARROW_RATIO,
                    // Numbers and most letters
                    '0'..='9' | 'a'..='z' | 'A'..='Z'
                    => FONT_SIZE * ASCII_NORMAL_RATIO,
                    // Other ASCII symbols
                    _ => FONT_SIZE * ASCII_NORMAL_RATIO,
                }
            },
            // Other Unicode characters, default half-width
            _ => FONT_SIZE * ASCII_NORMAL_RATIO,
        };

        total_width += char_width;
    }

    // Add left and right margins
    let padding: f64 = 173.0;
    let calculated_width = total_width + padding;

    // Limit width range: minimum 150, maximum 800
    let width = calculated_width.clamp(150.0, 800.0);

    LogicalSize::new(width, my_windows::window_translate_bubble::WINDOW_HEIGHT_TRANSLATE_BUBBLE)
}

// Optional: Support multiline text calculation
#[allow(dead_code)]
pub fn calculate_multiline_text_size(content: &str, max_width: f64) -> LogicalSize<f64> {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len().max(1);

    // Calculate the width of the widest line
    let max_line_width = lines
        .iter()
        .map(|line| {
            let size = calculate_text_width(line);
            size.width
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(150.0);

    let width = max_line_width.min(max_width);
    let height = my_windows::window_translate_bubble::WINDOW_HEIGHT_TRANSLATE_BUBBLE * line_count as f64;

    LogicalSize::new(width, height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_text() {
        let size = calculate_text_width("你好世界");
        // 4 Chinese characters × 14px + 100px padding = 156px
        assert!(size.width >= 150.0 && size.width <= 200.0);
    }

    #[test]
    fn test_mixed_text() {
        let size = calculate_text_width("Hello 世界");
        // Verify width is within reasonable range
        assert!(size.width >= 150.0 && size.width <= 800.0);
    }

    #[test]
    fn test_ascii_only() {
        let size = calculate_text_width("Hello World");
        assert!(size.width >= 150.0);
    }
}
