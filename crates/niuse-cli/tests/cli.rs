//! End-to-end checks for the binary's argument wiring.
//!
//! These run without a provider: status, memory writes, and JSON reads are all
//! offline. They catch the class of bug where a library function is correct but
//! the subcommand is wired to the wrong flag.

use std::process::Command;

fn run(home: &std::path::Path, args: &[&str]) -> std::process::Output {
    run_in(std::path::Path::new("."), home, args)
}

fn run_in(
    directory: &std::path::Path,
    home: &std::path::Path,
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_niuse"))
        .current_dir(directory)
        .arg("--home")
        .arg(home)
        .args(args)
        .output()
        .expect("run niuse")
}

#[test]
fn status_works_on_an_empty_home() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(&home, &["status"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("home"));
    assert!(stdout.contains("memory"));
    assert!(stdout.contains("user"));
}

#[test]
fn memory_add_and_json_list_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");

    let add = run(
        &home,
        &["mem", "add", "prefer short paragraphs", "--tags", "style"],
    );
    assert!(add.status.success(), "{add:?}");

    let list = run(&home, &["mem", "list", "--json"]);
    assert!(list.status.success(), "{list:?}");
    let entries: serde_json::Value = serde_json::from_slice(&list.stdout).expect("valid JSON");
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["text"], "prefer short paragraphs");
    assert_eq!(entries[0]["tags"][0], "style");
    assert_eq!(entries[0]["expired"], false);
}

#[test]
fn threads_json_is_an_empty_array_before_the_first_run() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(&home, &["threads", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let entries: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(entries, serde_json::json!([]));
}

#[test]
fn effective_config_reflects_cli_overrides() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(
        &home,
        &[
            "--sandbox",
            "read-only",
            "--max-steps",
            "3",
            "config",
            "effective",
        ],
    );
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sandbox     read-only"));
    assert!(stdout.contains("max_steps   3"));
}

#[test]
fn status_json_has_nested_sections() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(&home, &["status", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let status: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert!(status["version"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert_eq!(status["memory"]["total"], 0);
    assert_eq!(status["memory"]["preferences"], 0);
    assert_eq!(status["memory"]["lessons"], 0);
    assert_eq!(status["loops"]["open"], 0);
    assert!(status["prompt"]["tokens"].as_u64().unwrap() > 0);
}

#[test]
fn loops_json_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let add = run(
        &home,
        &["loops", "add", "check the pin", "--due", "2026-10-30"],
    );
    assert!(add.status.success(), "{add:?}");
    let output = run(&home, &["loops", "list", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let loops: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(loops.as_array().unwrap().len(), 1);
    assert_eq!(loops[0]["text"], "check the pin");
    assert_eq!(loops[0]["due"], "2026-10-30");
}

#[test]
fn memory_tag_filter_selects_matching_entries() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let first = run(
        &home,
        &["mem", "add", "prefer short paragraphs", "--tags", "style"],
    );
    assert!(first.status.success(), "{first:?}");
    let second = run(&home, &["mem", "add", "use nextest", "--tags", "rust"]);
    assert!(second.status.success(), "{second:?}");

    let list = run(&home, &["mem", "list", "--tag", "rust", "--json"]);
    assert!(list.status.success(), "{list:?}");
    let entries: serde_json::Value = serde_json::from_slice(&list.stdout).expect("valid JSON");
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["tags"][0], "rust");

    let search = run(
        &home,
        &["mem", "search", "nextest", "--tag", "style", "--json"],
    );
    assert!(search.status.success(), "{search:?}");
    let hits: serde_json::Value = serde_json::from_slice(&search.stdout).expect("valid JSON");
    assert_eq!(hits, serde_json::json!([]));
}

#[test]
fn relative_thread_path_is_marked_active() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(
        home.join("thread.jsonl"),
        "{\"type\":\"session_start\",\"id\":\"thread\",\"cwd\":\"/tmp\",\"model\":\"m\",\"provider\":\"p\",\"ts\":1}\n",
    )
    .unwrap();

    let output = run_in(
        &home,
        &home,
        &["--thread", "thread.jsonl", "threads", "--json"],
    );
    assert!(output.status.success(), "{output:?}");
    let threads: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(threads[0]["path"], "thread.jsonl");
    assert_eq!(threads[0]["active"], true);
}

#[test]
fn sessions_json_lists_exec_logs() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let sessions = home.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::write(
        sessions.join("run-1.jsonl"),
        "{\"type\":\"user_message\",\"text\":\"one shot\",\"ts\":1}\n",
    )
    .unwrap();

    let output = run(&home, &["sessions", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let entries: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(entries.as_array().unwrap().len(), 1);
    assert_eq!(entries[0]["title"], "one shot");
    assert_eq!(entries[0]["active"], false);
}

#[test]
fn corrupt_session_lines_are_tolerated() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let sessions = home.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    let log = sessions.join("bad.jsonl");
    std::fs::write(
        &log,
        "not json\n{\"type\":\"user_message\",\"text\":\"ok\",\"ts\":1}\n",
    )
    .unwrap();

    let listing = run(&home, &["sessions", "--json"]);
    assert!(listing.status.success(), "{listing:?}");
    let entries: serde_json::Value = serde_json::from_slice(&listing.stdout).expect("valid JSON");
    assert_eq!(entries[0]["title"], "ok");
    assert_eq!(entries[0]["events"], 1);

    let out = home.join("export.md");
    let export = run(
        &home,
        &[
            "export",
            "--session",
            log.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ],
    );
    assert!(export.status.success(), "{export:?}");
    let text = std::fs::read_to_string(&out).unwrap();
    assert!(text.starts_with("# ok\n"));
    assert!(text.contains("ok"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&out).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn completions_render_without_touching_the_home() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(&home, &["completions", "bash"]);
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("niuse"));
    assert!(!home.exists());
}

#[test]
fn overdue_filter_selects_late_loops() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let late = run(
        &home,
        &["loops", "add", "late promise", "--due", "2020-01-01"],
    );
    assert!(late.status.success(), "{late:?}");
    let future = run(
        &home,
        &["loops", "add", "future promise", "--due", "2099-01-01"],
    );
    assert!(future.status.success(), "{future:?}");

    let output = run(&home, &["loops", "list", "--overdue", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let loops: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(loops.as_array().unwrap().len(), 1);
    assert_eq!(loops[0]["text"], "late promise");
    assert_eq!(loops[0]["overdue"], true);
}

#[test]
fn effective_config_json_is_structured() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let output = run(&home, &["config", "effective", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let config: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(config["provider"], "deepseek");
    assert_eq!(config["sandbox"], "workspace-write");
    assert_eq!(config["max_steps"], 0);
}

#[test]
fn memory_show_json_round_trip() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let add = run(&home, &["mem", "add", "single entry", "--tags", "style"]);
    assert!(add.status.success(), "{add:?}");
    let list = run(&home, &["mem", "list", "--json"]);
    let entries: serde_json::Value = serde_json::from_slice(&list.stdout).expect("valid JSON");
    let id = entries[0]["id"].as_str().unwrap().to_string();

    let output = run(&home, &["mem", "show", &id, "--json"]);
    assert!(output.status.success(), "{output:?}");
    let entry: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(entry["id"], id);
    assert_eq!(entry["text"], "single entry");
    assert_eq!(entry["tags"][0], "style");
}
