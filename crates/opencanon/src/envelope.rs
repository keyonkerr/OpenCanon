use serde::Serialize;
use serde_json::Value;

use crate::error::CliError;
use crate::map_error::mapped;

#[derive(Serialize)]
struct OkEnvelope<'a> {
    ok: bool,
    command: &'a str,
    data: Value,
}

#[derive(Serialize)]
struct ErrEnvelope<'a> {
    ok: bool,
    command: &'a str,
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

pub fn write_ok(command: &str, data: Value) {
    let envelope = OkEnvelope {
        ok: true,
        command,
        data,
    };
    println!("{}", serde_json::to_string(&envelope).expect("envelope"));
}

pub fn write_err(command: &str, err: CliError) {
    let mapped = mapped(&err);
    let envelope = ErrEnvelope {
        ok: false,
        command,
        error: ErrorBody {
            code: mapped.code,
            message: mapped.message,
            details: mapped.details,
        },
    };
    println!("{}", serde_json::to_string(&envelope).expect("envelope"));
}
