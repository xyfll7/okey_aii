use selection;
use std::thread::sleep;
use std::time::Duration;


pub fn get_selected_text() -> String {
    let selected_text = selection::get_text();
    if !selected_text.is_empty() {
        return selected_text;
    }

    sleep(Duration::from_millis(100));

    selection::get_text()
}
