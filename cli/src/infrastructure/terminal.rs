//! # Terminal UI Utilities
//!
//! This module provides utilities for terminal-based user interface elements,
//! including colored output macros, application branding, and contextual help
//! messages. It enhances the user experience with formatted output and
//! visual feedback.

use crate::Args;
use clap::CommandFactory;
use colored::Colorize;

/// Macro for printing informational messages in bold bright white.
/// 
/// This macro formats and prints messages with emphasis, typically used
/// for important status updates or informational content.
#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().bright_white());
    };
}

/// Macro for printing error messages in bold red to stderr.
/// 
/// This macro formats and prints error messages with red coloring
/// to stderr, making them easily distinguishable from normal output.
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        eprintln!("{}", format!($($arg)*).bold().red());
    };
}

/// Macro for printing success messages in bold green.
/// 
/// This macro formats and prints success messages with green coloring
/// to indicate successful operations and positive outcomes.
#[macro_export]
macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().green());
    };
}

/// Macro for printing warning messages in bold yellow.
/// 
/// This macro formats and prints warning messages with yellow coloring
/// to indicate cautionary information or non-critical issues.
#[macro_export]
macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().yellow());
    };
}

/// Macro for clearing the terminal screen.
/// 
/// This macro sends ANSI escape sequences to clear the screen and
/// position the cursor at the top-left corner, providing a clean slate
/// for output display.
#[macro_export]
macro_rules! clear_screen {
    () => {
        print!("\x1B[2J\x1B[1;1H");
    };
}

/// Prints the application title banner with branding.
/// 
/// Displays a nicely formatted banner with the application name "Noir Stylus Verifier"
/// and branding attribution to WakeupLabs, enclosed in a decorative border.
pub(crate) fn print_app_title() {
    let title = "Noir Stylus Verifier";
    let circles = "◌○●";
    let powered_by = "Powered by WakeupLabs";

    let plain = format!("{title} {circles} {powered_by}");
    let content = format!(
        "{} {} {}",
        title.blue().bold(),
        circles.white(),
        powered_by.white()
    );

    let horizontal = "─".repeat(plain.len()).blue();
    let spacer = " ".repeat(plain.len()).blue();

    println!("\n{}{}{}", "╭".blue(), horizontal, "╮".blue());
    println!("{}{}{}", "│".blue(), spacer, "│".blue());
    println!("{}   {}   {}", "│".blue(), content, "│".blue());
    println!("{}{}{}", "│".blue(), spacer, "│".blue());
    println!("{}{}{}\n", "╰".blue(), horizontal, "╯".blue());
}

/// Prints contextual help instructions for suggested next commands.
/// 
/// Displays a "What's Next?" section with descriptions of suggested commands
/// that the user might want to run next, using the command descriptions from
/// the CLI argument parser.
/// 
/// # Arguments
/// 
/// * `commands` - Array of command names to display as suggestions
pub(crate) fn print_instructions(commands: &[&str]) {
    println!("\n\n  {}", "What's Next?\n".bright_white().bold());

    let mut cmd = Args::command();

    for command in commands {
        if let Some(sub) = cmd.find_subcommand_mut(command) {
            let about = sub.get_about().unwrap_or_default();
            println!(
                "    - {} {}: {about}",
                env!("CARGO_BIN_NAME").blue(),
                command.blue(),
            );
        }
    }

    println!();
}
