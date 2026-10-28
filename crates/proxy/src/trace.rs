use serde_json::{json, Value};
use std::io::Write;

pub fn event(kind: &str, details: Value) {
    let Some(destination) = std::env::var_os("ZED_CPP_DOCS_TRACE_HOVER") else {
        return;
    };
    let record = json!({"event": kind, "pid": std::process::id(), "details": details});
    if destination == "1" {
        eprintln!("C/C++ Docs trace: {record}");
    } else if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(destination)
    {
        let _ = writeln!(file, "{record}");
    }
}
