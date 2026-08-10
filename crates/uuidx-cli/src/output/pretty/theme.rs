use anstyle::{AnsiColor, Style};

fn paint(style: Style, value: &str) -> String {
    format!("{style}{value}{style:#}")
}

pub fn heading(value: &str) -> String {
    paint(
        Style::new()
            .bold()
            .fg_color(Some(AnsiColor::BrightCyan.into())),
        value,
    )
}

pub fn label(value: &str) -> String {
    paint(Style::new().fg_color(Some(AnsiColor::Cyan.into())), value)
}

pub fn version_tag(value: &str) -> String {
    paint(
        Style::new().bold().fg_color(Some(AnsiColor::Green.into())),
        value,
    )
}

pub fn success(value: &str) -> String {
    paint(
        Style::new()
            .bold()
            .fg_color(Some(AnsiColor::BrightGreen.into())),
        value,
    )
}

pub fn warning(value: &str) -> String {
    paint(
        Style::new()
            .bold()
            .fg_color(Some(AnsiColor::BrightYellow.into())),
        value,
    )
}

pub fn error(value: &str) -> String {
    paint(
        Style::new()
            .bold()
            .fg_color(Some(AnsiColor::BrightRed.into())),
        value,
    )
}

pub fn dim(value: &str) -> String {
    paint(Style::new().dimmed(), value)
}
