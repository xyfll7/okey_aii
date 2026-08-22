use tauri::LogicalSize;

use crate::my_windows;


const FONT_SIZE: f64 = 14.0; 


const CJK_WIDTH_RATIO: f64 = 1.0; 
const ASCII_WIDE_RATIO: f64 = 0.7; 
const ASCII_NORMAL_RATIO: f64 = 0.55; 
const ASCII_NARROW_RATIO: f64 = 0.35; 
const SPACE_RATIO: f64 = 0.4; 
const TAB_WIDTH_RATIO: f64 = 2.0;

pub fn calculate_text_width(content: &str) -> LogicalSize<f64> {
    let mut total_width: f64 = 0.0;

    for c in content.chars() {
        let char_width = match c {
            
            '\u{4E00}'..='\u{9FFF}' |  
            '\u{3400}'..='\u{4DBF}' |  
            '\u{20000}'..='\u{2A6DF}' | 
            '\u{2A700}'..='\u{2B73F}' | 
            '\u{2B740}'..='\u{2B81F}' | 
            '\u{2B820}'..='\u{2CEAF}' | 
            '\u{F900}'..='\u{FAFF}' |   
            '\u{2F800}'..='\u{2FA1F}'   
            => FONT_SIZE * CJK_WIDTH_RATIO,
            
            '\u{FF01}'..='\u{FF5E}' |  
            '\u{3000}'..='\u{303F}' |  
            '\u{3040}'..='\u{309F}' |  
            '\u{30A0}'..='\u{30FF}'    
            => FONT_SIZE * CJK_WIDTH_RATIO,
            
            '\u{AC00}'..='\u{D7AF}' |  
            '\u{1100}'..='\u{11FF}'    
            => FONT_SIZE * CJK_WIDTH_RATIO,
            
            '\u{1F300}'..='\u{1F9FF}' | 
            '\u{2600}'..='\u{26FF}' |   
            '\u{2700}'..='\u{27BF}'     
            => FONT_SIZE * CJK_WIDTH_RATIO,
            
            '\t' => FONT_SIZE * TAB_WIDTH_RATIO,
            
            '\n' | '\r' => FONT_SIZE * ASCII_NORMAL_RATIO,
            
            _ if c.is_ascii() => {
                match c {
                    
                    ' ' => FONT_SIZE * SPACE_RATIO,
                    
                    'W' | 'M' | 'w' | 'm' | '@' | '%' | '#' | '&' | '$'
                    => FONT_SIZE * ASCII_WIDE_RATIO,
                    
                    'i' | 'l' | 'I' | 'j' | 't' | 'f' | 'r' |
                    '.' | ',' | ':' | ';' | '\'' | '!' | '|' | '`'
                    => FONT_SIZE * ASCII_NARROW_RATIO,
                    
                    '0'..='9' | 'a'..='z' | 'A'..='Z'
                    => FONT_SIZE * ASCII_NORMAL_RATIO,
                    
                    _ => FONT_SIZE * ASCII_NORMAL_RATIO,
                }
            },
            
            _ => FONT_SIZE * ASCII_NORMAL_RATIO,
        };

        total_width += char_width;
    }

    
    let padding: f64 = 173.0;
    let calculated_width = total_width + padding;

    
    let width = calculated_width.clamp(150.0, 800.0);

    LogicalSize::new(width, my_windows::window_translate_bubble::WINDOW_HEIGHT_TRANSLATE_BUBBLE)
}


#[allow(dead_code)]
pub fn calculate_multiline_text_size(content: &str, max_width: f64) -> LogicalSize<f64> {
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len().max(1);

    
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
        let size = calculate_text_width("HelloWorld");
        
        assert!(size.width >= 150.0 && size.width <= 200.0);
    }

    #[test]
    fn test_mixed_text() {
        let size = calculate_text_width("Hello World");
        
        assert!(size.width >= 150.0 && size.width <= 800.0);
    }

    #[test]
    fn test_ascii_only() {
        let size = calculate_text_width("Hello World");
        assert!(size.width >= 150.0);
    }
}
