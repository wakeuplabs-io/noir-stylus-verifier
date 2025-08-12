//! # Progress Indicators
//!
//! This module provides progress indication utilities for long-running operations.
//! It creates spinner-style progress indicators that provide visual feedback to
//! users during CLI operations like compilation, deployment, and proof generation.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a spinner-style progress indicator with a custom message.
/// 
/// This function creates a visual spinner that rotates continuously to indicate
/// ongoing operations. The spinner is automatically hidden during testing to
/// avoid interfering with test output.
/// 
/// # Arguments
/// 
/// * `message` - The message to display alongside the spinner
/// 
/// # Returns
/// 
/// Returns a configured `ProgressBar` instance that displays a blue spinner
/// with the provided message. The spinner updates every 10 milliseconds.
/// 
/// # Examples
/// 
/// ```rust
/// let spinner = create_spinner("Compiling circuit...");
/// // ... perform long operation ...
/// spinner.finish_and_clear();
/// ```
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
