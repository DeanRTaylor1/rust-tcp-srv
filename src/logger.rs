use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Debug,
    Warning,
    Error,
}

pub struct Logger {
    dev_mode: bool,
}

#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Unknown,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

struct ColorCode(&'static str);

impl ColorCode {
    const BG_BLACK: ColorCode = ColorCode("\x1b[40m");
    const BG_RED: ColorCode = ColorCode("\x1b[41m");
    const BG_GREEN: ColorCode = ColorCode("\x1b[42m");
    const BG_YELLOW: ColorCode = ColorCode("\x1b[43m");
    const BG_BLUE: ColorCode = ColorCode("\x1b[44m");
    const BG_MAGENTA: ColorCode = ColorCode("\x1b[45m");
    const BG_CYAN: ColorCode = ColorCode("\x1b[46m");
    const FG_RED: ColorCode = ColorCode("\x1b[31m");
    const FG_GREEN: ColorCode = ColorCode("\x1b[32m");
    const FG_YELLOW: ColorCode = ColorCode("\x1b[33m");
    const FG_BLUE: ColorCode = ColorCode("\x1b[34m");
    const RESET: ColorCode = ColorCode("\x1b[0m");
}

impl Logger {
    pub fn new(dev_mode: bool) -> Self {
        Self { dev_mode }
    }

    fn format_status(&self, status: u16) -> String {
        if !self.dev_mode {
            return format!(" {} ", status);
        }

        let color = match status {
            s if s >= 500 => ColorCode::BG_RED,
            s if s >= 400 => ColorCode::BG_YELLOW,
            s if s >= 300 => ColorCode::BG_CYAN,
            s if s >= 200 => ColorCode::BG_GREEN,
            _ => ColorCode::BG_BLACK,
        };

        format!("{} {} {}", color.0, status, ColorCode::RESET.0)
    }

    fn format_method(&self, method: HttpMethod) -> String {
        if !self.dev_mode {
            return format!("{}", method);
        }

        let (color, padding) = match method {
            HttpMethod::Get => (ColorCode::BG_BLUE, "      "),
            HttpMethod::Post => (ColorCode::BG_CYAN, "     "),
            HttpMethod::Put => (ColorCode::BG_YELLOW, "      "),
            HttpMethod::Patch => (ColorCode::BG_MAGENTA, "    "),
            HttpMethod::Delete => (ColorCode::BG_RED, "   "),
            HttpMethod::Unknown => (ColorCode::BG_BLACK, ""),
        };

        format!("{} {}{} =>{}", color.0, method, padding, ColorCode::RESET.0)
    }

    pub fn log_request(&self, method: HttpMethod, path: &str, status: u16) {
        let method_str = self.format_method(method);
        let status_str = self.format_status(status);
        println!("{} {} {}", method_str, path, status_str);
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if !self.dev_mode {
            println!("[{}] {}", level.as_str(), message);
            return;
        }

        let (bg_color, fg_color, label) = match level {
            LogLevel::Info => (ColorCode::BG_GREEN, ColorCode::FG_GREEN, "INFO "),
            LogLevel::Debug => (ColorCode::BG_BLUE, ColorCode::FG_BLUE, "DEBUG"),
            LogLevel::Warning => (ColorCode::BG_YELLOW, ColorCode::FG_YELLOW, "WARN "),
            LogLevel::Error => (ColorCode::BG_RED, ColorCode::FG_RED, "ERROR"),
        };

        println!(
            "{} {} {} {} {} {}",
            bg_color.0,
            label,
            ColorCode::RESET.0,
            fg_color.0,
            message,
            ColorCode::RESET.0
        );
    }
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}
