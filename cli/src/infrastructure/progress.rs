use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub(crate) fn create_spinner(message: &str) -> ProgressBar {
    let spinner = if cfg!(test) {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };

    spinner.set_style(ProgressStyle::with_template("{spinner:.blue} {msg}").unwrap());
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(10));
    spinner.tick();

    spinner
}
