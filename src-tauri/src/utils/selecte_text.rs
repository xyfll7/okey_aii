use selection;
use serde::Serialize;
use std::thread::sleep;
use std::time::Duration;

/// 一次全局“选中”操作的结果：被选中的文本 + Finder 中被选中的文件路径。
#[derive(Debug, Clone, Serialize)]
pub struct SelectedContent {
    pub selected_text: String,
    pub selected_files: Vec<String>,
}

pub fn get_selected_content() -> SelectedContent {
    // 测试用：同时读取 Finder 中选中的文件路径并打印
    let selected_files = crate::utils::selected_files::get_selected_file_paths();
    log::info!("[test] selected_files => {:?}", selected_files);

    let selected_text = selection::get_text();
    if !selected_text.is_empty() {
        return SelectedContent {
            selected_text,
            selected_files,
        };
    }

    sleep(Duration::from_millis(100));

    SelectedContent {
        selected_text: selection::get_text(),
        selected_files,
    }
}
