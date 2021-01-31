use std::str::FromStr;

use log::LevelFilter;

#[derive(Debug, PartialEq, Eq)]
pub enum LogLevel {
    Silent,
    Debug,
    Info,
    Error,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "debug" | "all" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "error" => Ok(Self::Error),
            "silent" | "quiet" => Ok(Self::Silent),
            other => {
                let error_msg = format!(
                    "Cannot get log level for: {:?}. Using default log level.",
                    other
                )
                .to_owned();
                Err(error_msg)
            }
        }
    }
}

impl Into<&'static str> for LogLevel {
    fn into(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Error => "error",
            Self::Silent => "silent",
        }
    }
}

impl From<&LogLevel> for LevelFilter {
    fn from(l: &LogLevel) -> Self {
        match l {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Silent => LevelFilter::Off,
        }
    }
}
impl From<&LogLevel> for String {
    fn from(log: &LogLevel) -> Self {
        match log {
            LogLevel::Debug => "debug".to_owned(),
            LogLevel::Info => "info".to_owned(),
            LogLevel::Error => "error".to_owned(),
            LogLevel::Silent => "silent".to_owned(),
        }
    }
}
