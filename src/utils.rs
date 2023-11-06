use owo_colors::OwoColorize;
use std::time::Duration;

pub fn strip_ansi(string: &str) -> String {
    let re = regex::Regex::new(r"\x1B\[([0-9]{1,2}(;[0-9]{1,2})?)?[m|K]").unwrap();
    re.replace_all(string, "").to_string()
}

pub fn colorize_number(number: &usize) -> String {
    let number_str = number.to_string();
    let number_str_len = number_str.len();

    let mut colored_number = String::new();

    // more than 100, red
    // more than 10, yellow
    // more than 1, green
    if number_str_len > 2 {
        colored_number.push_str(&number_str.red().bold().to_string());
    } else if number_str_len > 1 {
        colored_number.push_str(&number_str.yellow().bold().to_string());
    } else {
        colored_number.push_str(&number_str.green().bold().to_string());
    }

    colored_number
}

pub fn get_time_unit(time: &Duration) -> String {
    if time.as_secs() < 60 {
        format!("{}s", time.as_secs())
    } else if time.as_secs() < 3600 {
        format!("{}m", time.as_secs() / 60)
    } else if time.as_secs() < 86400 {
        format!("{}h", time.as_secs() / 3600)
    } else {
        format!("{}d", time.as_secs() / 86400)
    }
}

pub fn colorize_time(time: &Duration) -> String {
    let time_str = get_time_unit(time);

    let mut colored_time = String::new();

    // more than 60s, red
    // more than 10s, yellow
    // more than 1s, green

    if time.as_secs() > 60 {
        colored_time.push_str(&time_str.red().to_string());
    } else if time.as_secs() > 10 {
        colored_time.push_str(&time_str.yellow().to_string());
    } else {
        colored_time.push_str(&time_str.green().to_string());
    }

    colored_time
}
