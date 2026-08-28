use rig::message::{Message, UserContent};
use tauri::Manager;

use crate::ai::state::HistoryItem;
use crate::store::app_config::Language;
use crate::store::app_state::AppConfigState;
use crate::utils::language_detection;

pub fn assemble_prompt_item(app: &tauri::AppHandle, mut item: HistoryItem) -> HistoryItem {
    let Message::User { content } = &item.message else {
        return item;
    };

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
