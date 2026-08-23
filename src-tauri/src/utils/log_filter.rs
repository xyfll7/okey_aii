use log::{Level, Metadata};

pub fn log_filter(metadata: &Metadata) -> bool {
    if metadata.level() == Level::Warn && metadata.target() == "tao::platform_impl::platform::event_loop::runner" {
        false
    } else {
        true
    }
}
