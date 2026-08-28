use crate::my_tray;
use crate::store::app_config::Language;
use crate::store::app_state::AppConfigState;

#[tauri::command]
pub fn get_current_locale() -> String {
    rust_i18n::locale().to_string()
}

#[tauri::command]
pub fn set_current_locale(app_handle: tauri::AppHandle, locale: String, app_config_state: tauri::State<AppConfigState>) {
    rust_i18n::set_locale(&locale);

    let language = Language::from_locale(&locale);

    let _ = app_config_state.update_and_save(|config| {
        config.language = language;
    });

    let _ = my_tray::rebuild_tray_menu(&app_handle);
}

pub fn get_default_locale() -> String {
    let lang = tauri_plugin_os::locale()
        .map(|full_locale| {
            let prefix = full_locale.split('-').next().unwrap_or("");
            if prefix.eq_ignore_ascii_case("zh") {
                "zh-CN".to_string()
            } else {
                "en".to_string()
            }
        })
        .unwrap_or_else(|| "en".to_string());
    lang
}
