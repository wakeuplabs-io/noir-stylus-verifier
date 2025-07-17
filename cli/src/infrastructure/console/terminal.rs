use colored::Colorize;

#[allow(dead_code)]
pub(crate) fn print_info(msg: &str) {
    println!("{}", msg.bright_white().bold());
}

#[allow(dead_code)]
pub(crate) fn print_error(msg: &str) {
    eprintln!("{}", msg.bold().red());
}

#[allow(dead_code)]
pub(crate) fn print_success(msg: &str) {
    println!("{}", msg.bold().green());
}

#[allow(dead_code)]
pub(crate) fn print_warning(msg: &str) {
    println!("{}", msg.bold().yellow());
}
