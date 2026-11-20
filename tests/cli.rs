use std::fs;
use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_phone-region-cli"))
}

fn json_stdout(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be a JSON document")
}

fn json_stderr(output: &Output) -> Value {
    serde_json::from_slice(&output.stderr).expect("stderr should be a JSON document")
}

#[test]
fn lookup_returns_stable_order_and_distinguishes_unknown_numbers() {
    let output = cli()
        .args([
            "lookup",
            "+86 132 1300 0000",
            "13447100000",
            "not a phone",
            "  ",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());

    let body = json_stdout(&output);
    assert_eq!(body["schema_version"], 1);
    assert_eq!(
        body["results"],
        json!([
            {
                "status": "found",
                "phone": "13213000000",
                "carrier": "中国联通",
                "province": "河南省",
                "city": "郑州市"
            },
            {
                "status": "missing",
                "phone": "13447100000",
                "carrier": null,
                "province": null,
                "city": null
            },
            {
                "status": "invalid",
                "phone": "not a phone",
                "carrier": null,
                "province": null,
                "city": null
            },
            {
                "status": "empty",
                "phone": "  ",
                "carrier": null,
                "province": null,
                "city": null
            }
        ])
    );
}

#[test]
fn stdin_accepts_json_strings_and_reports_malformed_requests() {
    let mut child = cli()
        .args(["lookup", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"["13213000000","a \"quoted\" value",""]"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let result = json_stdout(&output);
    assert_eq!(result["results"].as_array().unwrap().len(), 3);
    assert_eq!(result["results"][1]["phone"], "a \"quoted\" value");
    assert_eq!(result["results"][1]["status"], "invalid");

    let mut child = cli()
        .args(["lookup", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(br#"[123]"#).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let error = json_stderr(&output);
    assert_eq!(error["schema_version"], 1);
    assert!(
        error["error"]["code"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
    assert!(
        error["error"]["message"]
            .as_str()
            .is_some_and(|s| !s.is_empty())
    );
}

#[test]
fn batch_exports_without_replacing_existing_files() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.csv");
    let output = directory.path().join("output.csv");
    fs::write(&input, "手机号\n13213000000\n13447100000\n").unwrap();

    let first = cli()
        .args(["batch", input.to_str().unwrap(), output.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(first.status.code(), Some(0));
    assert!(first.stderr.is_empty());
    let body = json_stdout(&first);
    assert_eq!(body["schema_version"], 1);
    assert_eq!(body["rows"], 2);
    assert_eq!(body["output"], output.to_str().unwrap());
    let bytes = fs::read(&output).unwrap();
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
    assert!(String::from_utf8_lossy(&bytes).contains("13213000000,中国联通,河南省,郑州市"));

    let second = cli()
        .args(["batch", input.to_str().unwrap(), output.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    assert!(second.stdout.is_empty());
    assert!(json_stderr(&second)["error"]["message"].is_string());
    assert_eq!(fs::read(&output).unwrap(), bytes);
}

#[test]
fn batch_accepts_output_filename_relative_to_working_directory() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("input.csv");
    fs::write(&input, "手机号\n13213000000\n").unwrap();

    let output = cli()
        .current_dir(directory.path())
        .args(["batch", input.to_str().unwrap(), "result.csv"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    assert_eq!(json_stdout(&output)["rows"], 1);
    let bytes = fs::read(directory.path().join("result.csv")).unwrap();
    assert!(bytes.starts_with(&[0xef, 0xbb, 0xbf]));
}

#[test]
fn help_version_and_capabilities_do_not_need_input() {
    for option in ["--help", "--version"] {
        let output = cli().arg(option).stdin(Stdio::null()).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{option}");
        assert!(!output.stdout.is_empty(), "{option}");
    }
    let describe = cli().arg("describe").stdin(Stdio::null()).output().unwrap();
    assert_eq!(describe.status.code(), Some(0));
    let body = json_stdout(&describe);
    assert_eq!(body["schema_version"], 1);
    assert!(body["commands"].is_array() || body["commands"].is_object());

    let no_input = cli()
        .args(["lookup", "--stdin"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(no_input.status.code(), Some(2));
    assert!(no_input.stdout.is_empty());
    assert!(json_stderr(&no_input)["error"]["code"].is_string());
}
