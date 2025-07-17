use colored::Colorize;

#[macro_export]
macro_rules! print_info {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().bright_white());
    };
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        eprintln!("{}", format!($($arg)*).bold().red());
    };
}

#[macro_export]
macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().green());
    };
}

#[macro_export]
macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}", format!($($arg)*).bold().yellow());
    };
}

#[macro_export]
macro_rules! clear_screen {
    () => {
        print!("\x1B[2J\x1B[1;1H");
    };
}

pub(crate) fn print_app_title() {
    let title = "Noir Stylus Verifier";
    let circles = "◌○●";
    let powered_by = "Powered by WakeupLabs";

    let plain = format!("{} {} {}", title, circles, powered_by);
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
