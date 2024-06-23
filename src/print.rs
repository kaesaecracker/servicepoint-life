use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use std::io::stdout;

pub fn println_info(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().white()))
}

pub fn println_debug(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().grey()))
}

pub fn println_warning(text: impl Into<String>) {
    println_command(PrintStyledContent(text.into().red()))
}

pub fn println_command(command: impl crossterm::Command) {
    queue!(stdout(), command, Print("\r\n")).expect("could not print");
}
