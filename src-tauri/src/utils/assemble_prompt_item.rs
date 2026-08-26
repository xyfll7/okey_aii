use rig::message::{Message, UserContent};
use tauri::Manager;

use crate::ai::state::HistoryItem;
use crate::store::app_config::Language;
use crate::store::app_state::AppConfigState;
use crate::utils::language_detection;

/// 组装用户 prompt 中的模板（语言逻辑迁移自旧版 `assemble_prompt`）。
///
/// 约定 User 消息 content 中第一条 Text 为用户选中的原文（raw），
/// 第二条 Text 为 prompt 模板；模板里的 `{text}` / `{detected_lang}` /
/// `{target}` / `{local}` 占位符会按语言检测结果与用户语言配置替换，
/// 组装结果回填到第二条 Text，其余 content 保持不变。
/// 非 User 消息或 Text 不足两条时原样返回。
pub fn assemble_prompt_item(app: &tauri::AppHandle, mut item: HistoryItem) -> HistoryItem {
    let Message::User { content } = &item.message else {
        return item;
    };

    // 第一条 Text 作为原文（raw），第二条 Text 作为模板
    let mut texts = content
        .iter()
        .filter_map(|c| match c {
            UserContent::Text(t) => Some(t.text.clone()),
            _ => None,
        });
    let (Some(raw), Some(template)) = (texts.next(), texts.next()) else {
        return item;
    };

    let app_config_state = app.state::<AppConfigState>();
    let app_config_read = app_config_state.read();

    let detected_language = if raw.is_empty() {
        None
    } else {
        Some(Language::from_locale(
            language_detection::detect_language(&raw),
        ))
    };
    let detected_lang = detected_language
        .as_ref()
        .map(|lang| lang.to_display_name().to_string())
        .unwrap_or_default();

    let effective_local_language = app_config_read.local_language.effective_language();
    let effective_target_language = app_config_read.target_language.effective_language();
    let local = effective_local_language.to_display_name().to_string();

    let target_lang = match detected_language {
        Some(detected) if detected == effective_local_language => effective_target_language,
        Some(_) => effective_local_language,
        None => effective_target_language,
    }
    .to_display_name()
    .to_string();

    #[rustfmt::skip]
    let assembled = template
        .replace("{text}", &raw)
        .replace("{detected_lang}", &detected_lang)
        .replace("{target}", &target_lang)
        .replace("{local}", &local);

    // 将第二条 Text（模板）替换为组装结果，其余 content 保持不变
    if let Message::User { content } = &mut item.message {
        let mut text_idx = 0usize;
        for c in content.iter_mut() {
            if let UserContent::Text(t) = c {
                if text_idx == 1 {
                    t.text = assembled;
                    break;
                }
                text_idx += 1;
            }
        }
    }
    item
}
