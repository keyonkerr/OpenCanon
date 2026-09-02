use std::env;

use canon_core::Timestamp;
use chrono::{Datelike, Local, NaiveDateTime, Timelike};

use crate::error::CliError;

/// Injected clock. `OPENCANON_NOW=YYYY-MM-DD HH:MM:SS` is a test-only override.
pub fn resolve_now() -> Result<Timestamp, CliError> {
    match env::var("OPENCANON_NOW") {
        Ok(raw) => parse_now(&raw),
        Err(_) => {
            let now = Local::now().naive_local();
            Ok(Timestamp::from_ymd_hms(
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
            ))
        }
    }
}

fn parse_now(raw: &str) -> Result<Timestamp, CliError> {
    let normalized = raw.replace('T', " ");
    let dt = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S").map_err(|_| {
        CliError::Io {
            message: format!("invalid OPENCANON_NOW `{raw}`, expected YYYY-MM-DD HH:MM:SS"),
        }
    })?;
    Ok(Timestamp::from_ymd_hms(
        dt.year(),
        dt.month(),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
    ))
}
