/// Enum representing all available translation keys
/// This provides type safety and IDE autocomplete support
#[derive(Debug, Clone)]
pub enum TRKey {
    Quit,
    Show,
    Test,
    Autostart,
    NotificationTitle,
    NotificationBody,
}

impl TRKey {
    fn as_str(&self) -> &'static str {
        match self {
            TRKey::Quit => "quit",
            TRKey::Show => "show",
            TRKey::Test => "test",
            TRKey::Autostart => "autostart",
            TRKey::NotificationTitle => "notification_title",
            TRKey::NotificationBody => "notification_body",
        }
    }

    /// Get the translated string for this key
    pub fn t(&self) -> String {
        rust_i18n::t!(self.as_str()).to_string()
    }
}
