use std::io::Write;
use std::process::{Command, Stdio};

fn run_filter(input: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_opensmtpd-filter-gglgrp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start filter");

    child.stdin.take().unwrap().write_all(input).unwrap();

    let output = child.wait_with_output().expect("failed to wait on filter");

    (output.stdout, output.stderr)
}

#[test]
fn config_ready_registers_events() {
    let (stdout, _) = run_filter(b"config|ready\n");
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("register|report|smtp-in|tx-data\n"));
    assert!(stdout.contains("register|filter|smtp-in|data-line\n"));
    assert!(stdout.contains("register|filter|smtp-in|commit\n"));
    assert!(stdout.contains("register|report|smtp-in|link-disconnect\n"));
    assert!(stdout.contains("register|ready\n"));
}

#[test]
fn data_line_is_passed_through() {
    let input = b"config|ready\n\
                  report|0.7|1|smtp-in|tx-data|abc123\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok1|Subject: Test\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok2|.\n";
    let (stdout, _) = run_filter(input);
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("filter-dataline|abc123|tok1|Subject: Test\n"));
    assert!(stdout.contains("filter-dataline|abc123|tok2|.\n"));
}

#[test]
fn commit_allows_non_google_group_email() {
    let input = b"config|ready\n\
                  report|0.7|1|smtp-in|tx-data|abc123\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok1|Subject: Test\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok2|.\n\
                  filter|0.7|1|smtp-in|commit|abc123|tok3\n";
    let (stdout, _) = run_filter(input);
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("filter-result|abc123|tok3|proceed\n"));
}

#[test]
fn commit_rejects_google_group_email() {
    let input = b"config|ready\n\
                  report|0.7|1|smtp-in|tx-data|abc123\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok1|X-Google-Group-Id: 12345\n\
                  filter|0.7|1|smtp-in|data-line|abc123|tok2|.\n\
                  filter|0.7|1|smtp-in|commit|abc123|tok3\n";
    let (stdout, _) = run_filter(input);
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("filter-result|abc123|tok3|reject|550 Google Groups not allowed\n"));
}

#[test]
fn commit_allows_without_session() {
    let input = b"config|ready\n\
                  filter|0.7|1|smtp-in|commit|abc123|tok1\n";
    let (stdout, _) = run_filter(input);
    let stdout = String::from_utf8(stdout).unwrap();
    assert!(stdout.contains("filter-result|abc123|tok1|proceed\n"));
}
