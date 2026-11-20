use std::env;
use std::ffi::OsString;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::ExitCode;

use phone_region::batch;
use phone_region::lookup::{Database, Lookup, Status};
use serde_json::{Value, json};

const HELP: &str = "\
Offline mainland China mobile number lookup

Usage:
  phone-region-cli lookup NUMBER...
  phone-region-cli lookup --stdin       Read a UTF-8 JSON array of strings
  phone-region-cli batch INPUT OUTPUT   Write a new XLSX or CSV file
  phone-region-cli describe             Print the machine-readable CLI contract
  phone-region-cli --help | --version

Successful commands write JSON to stdout (help and version are plain text).
Errors write JSON to stderr. Exit codes: 0 completed, 1 runtime error, 2 invalid request.
";

const SCHEMA_VERSION: u8 = 1;

enum Failure {
    Request(String),
    Runtime(String),
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            let (exit, code, message) = match failure {
                Failure::Request(message) => (2, "invalid_request", message),
                Failure::Runtime(message) => (1, "operation_failed", message),
            };
            let error = json!({
                "schema_version": SCHEMA_VERSION,
                "error": {"code": code, "message": message}
            });
            let _ = writeln!(io::stderr().lock(), "{error}");
            ExitCode::from(exit)
        }
    }
}

fn run(args: &[OsString]) -> Result<(), Failure> {
    let mut stdout = io::stdout().lock();
    let result = match args.split_first().map(|(cmd, rest)| (cmd.to_str(), rest)) {
        Some((Some("--help" | "-h"), [])) => return write_text(&mut stdout, HELP),
        Some((Some("--version" | "-V"), [])) => {
            return write_text(
                &mut stdout,
                concat!("phone-region-cli ", env!("CARGO_PKG_VERSION"), "\n"),
            );
        }
        Some((Some("describe"), [])) => describe(),
        Some((Some("lookup"), args)) => {
            let numbers = lookup_inputs(args)?;
            let db = Database::load().map_err(Failure::Runtime)?;
            let results: Vec<_> = numbers
                .iter()
                .map(|number| record(db.lookup(number)))
                .collect();
            json!({"schema_version": SCHEMA_VERSION, "results": results})
        }
        Some((Some("batch"), [input, output])) => {
            let db = Database::load().map_err(Failure::Runtime)?;
            let rows = batch::prepare(Path::new(input), &db).map_err(Failure::Runtime)?;
            batch::export_prepared(&rows, Path::new(output)).map_err(Failure::Runtime)?;
            json!({"schema_version": SCHEMA_VERSION, "rows": rows.len(), "output": output.to_string_lossy()})
        }
        _ => {
            return Err(Failure::Request(
                "Unknown command or incorrect arguments. Run phone-region-cli --help.".into(),
            ));
        }
    };
    serde_json::to_writer(&mut stdout, &result).map_err(|e| Failure::Runtime(e.to_string()))?;
    write_text(&mut stdout, "\n")
}

fn write_text(writer: &mut impl Write, text: &str) -> Result<(), Failure> {
    writer
        .write_all(text.as_bytes())
        .and_then(|()| writer.flush())
        .map_err(|e| Failure::Runtime(e.to_string()))
}

fn lookup_inputs(args: &[OsString]) -> Result<Vec<String>, Failure> {
    if matches!(args, [flag] if flag == "--stdin") {
        let mut bytes = Vec::new();
        io::stdin()
            .lock()
            .read_to_end(&mut bytes)
            .map_err(|e| Failure::Runtime(e.to_string()))?;
        let input = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
        return serde_json::from_slice(input).map_err(|e| {
            Failure::Request(format!(
                "Expected a UTF-8 JSON array of phone number strings on stdin: {e}"
            ))
        });
    }
    if args.is_empty() {
        return Err(Failure::Request(
            "Supply at least one number, or use lookup --stdin.".into(),
        ));
    }
    args.iter()
        .map(|arg| match arg.to_str() {
            Some(s) if !s.starts_with("--") => Ok(s.to_owned()),
            _ => Err(Failure::Request(
                "Pass numbers as text arguments, or supply a JSON array with lookup --stdin."
                    .into(),
            )),
        })
        .collect()
}

fn record(row: Lookup) -> Value {
    let status = match row.status {
        Status::Found => "found",
        Status::Missing => "missing",
        Status::Invalid => "invalid",
        Status::Empty => "empty",
    };
    json!({
        "status": status,
        "phone": row.phone,
        "carrier": nonempty(row.carrier),
        "province": nonempty(row.province),
        "city": nonempty(row.city),
    })
}

fn nonempty(value: String) -> Option<String> {
    (!value.is_empty()).then_some(value)
}

fn describe() -> Value {
    json!({
        "name": "phone-region-cli",
        "homepage": "https://github.com/TaylorChyi/PhoneRegion",
        "version": env!("CARGO_PKG_VERSION"),
        "schema_version": SCHEMA_VERSION,
        "description": "Offline lookup of the original carrier, province and prefecture of mainland Chinese mobile numbers.",
        "commands": {
            "lookup": {
                "usage": ["lookup NUMBER...", "lookup --stdin"],
                "stdin": {"type": "array", "items": {"type": "string"}, "encoding": "UTF-8 JSON"},
                "stdout": {"results": "array in input order, including duplicates and unresolved numbers"},
                "fields": {
                    "status": ["found", "invalid", "missing", "empty"],
                    "phone": "string, normalised when valid",
                    "carrier": "original prefix carrier or null",
                    "province": "province or null",
                    "city": "prefecture-level area or null"
                }
            },
            "batch": {
                "usage": "batch INPUT OUTPUT",
                "input": ["xls", "xlsx", "csv"],
                "output": ["xlsx", "csv"],
                "overwrite": false,
                "stdout": {"rows": "number of exported data rows", "output": "output path"}
            },
            "describe": {"usage": "describe", "stdout": "CLI capabilities and response contract"}
        },
        "exit_codes": {"0": "completed", "1": "operation_failed", "2": "invalid_request"},
        "errors": {"stream": "stderr", "fields": ["schema_version", "error.code", "error.message"]}
    })
}
