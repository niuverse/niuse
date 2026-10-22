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

fn run_with_env(
    home: &std::path::Path,
    env: &[(&str, &str)],
    args: &[&str],
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_niuse"))
        .current_dir(".")
        .envs(env.iter().copied())
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
    assert!(status.get("project").is_some());
}

#[test]
fn brief_json_reports_actionable_context() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");

    let add = run(&home, &["mem", "add", "prefer short paragraphs"]);
    assert!(add.status.success(), "{add:?}");
    let promise = run(
        &home,
        &["loops", "add", "ship the brief", "--due", "2020-01-01"],
    );
    assert!(promise.status.success(), "{promise:?}");

    let output = run(&home, &["brief", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let brief: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(brief["memory"]["total"], 1);
    assert_eq!(brief["loops"]["open"], 1);
    assert_eq!(brief["loops"]["overdue"].as_array().unwrap().len(), 1);
    assert_eq!(brief["loops"]["overdue"][0]["text"], "ship the brief");
    assert!(brief["project"]
        .as_str()
        .is_some_and(|project| !project.is_empty()));

    let text = run(&home, &["brief"]);
    assert!(text.status.success(), "{text:?}");
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("niuse brief"));
    assert!(stdout.contains("ship the brief"));
}

#[test]
fn goals_round_trip_through_status() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");

    let added = run(&home, &["goals", "add", "ship the personal agent"]);
    assert!(added.status.success(), "{added:?}");
    let listed = run(&home, &["goals", "list", "--json"]);
    assert!(listed.status.success(), "{listed:?}");
    let goals: serde_json::Value = serde_json::from_slice(&listed.stdout).expect("valid JSON");
    assert_eq!(goals.as_array().unwrap().len(), 1);
    assert_eq!(goals[0]["text"], "ship the personal agent");
    let id = goals[0]["id"].as_str().unwrap().to_string();

    let status = run(&home, &["status", "--json"]);
    let status: serde_json::Value = serde_json::from_slice(&status.stdout).expect("valid JSON");
    assert_eq!(status["goals"]["open"], 1);

    let done = run(&home, &["goals", "done", &id]);
    assert!(done.status.success(), "{done:?}");
    let listed = run(&home, &["goals", "list", "--json"]);
    let goals: serde_json::Value = serde_json::from_slice(&listed.stdout).expect("valid JSON");
    assert_eq!(goals.as_array().unwrap().len(), 0);
}

#[test]
fn recall_finds_memory_journal_loops_and_goals() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");

    let added = run(
        &home,
        &[
            "mem",
            "add",
            "Rust projects use cargo nextest",
            "--tags",
            "rust",
        ],
    );
    assert!(added.status.success(), "{added:?}");
    let promise = run(&home, &["loops", "add", "finish the rust recall path"]);
    assert!(promise.status.success(), "{promise:?}");
    let journal = run(
        &home,
        &["journal", "--add", "decided the rust recall design"],
    );
    assert!(journal.status.success(), "{journal:?}");
    let goal = run(&home, &["goals", "add", "finish the rust goal"]);
    assert!(goal.status.success(), "{goal:?}");

    let output = run(&home, &["recall", "rust", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let recall: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    let hits = recall["hits"].as_array().unwrap();
    let kinds: Vec<&str> = hits.iter().filter_map(|hit| hit["kind"].as_str()).collect();
    assert!(kinds.contains(&"memory"));
    assert!(kinds.contains(&"journal"));
    assert!(kinds.contains(&"loop"));
    assert!(kinds.contains(&"goal"));

    let text = run(&home, &["recall", "rust"]);
    assert!(text.status.success(), "{text:?}");
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("recall \"rust\""));
    assert!(stdout.contains("cargo nextest"));
}

#[test]
fn evidence_json_reports_files_and_checks() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let thread = directory.path().join("thread.jsonl");
    std::fs::write(
        &thread,
        concat!(
            "{\"type\":\"assistant_message\",\"text\":\"working\",\"tool_calls\":[",
            "{\"id\":\"c1\",\"name\":\"write\",\"input\":{\"path\":\"src/lib.rs\",\"content\":\"x\"}},",
            "{\"id\":\"c2\",\"name\":\"bash\",\"input\":{\"command\":\"cargo test\"}}",
            "],\"ts\":1}\n",
            "{\"type\":\"tool_result\",\"call_id\":\"c1\",\"ok\":true,\"output\":\"ok\",\"ts\":2}\n",
            "{\"type\":\"tool_result\",\"call_id\":\"c2\",\"ok\":true,\"output\":\"ok\",\"ts\":3}\n",
        ),
    )
    .unwrap();

    let output = run(
        &home,
        &["evidence", "--thread", thread.to_str().unwrap(), "--json"],
    );
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(report["tools"], 2);
    assert_eq!(report["files"][0], "src/lib.rs");
    assert_eq!(report["checks"][0]["command"], "cargo test");
    assert_eq!(report["checks"][0]["ok"], true);

    let text = run(&home, &["evidence", "--thread", thread.to_str().unwrap()]);
    assert!(text.status.success(), "{text:?}");
    assert!(String::from_utf8_lossy(&text.stdout).contains("evidence"));
}

#[test]
fn verify_records_and_evidence_reports_it() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let thread = directory.path().join("thread.jsonl");
    std::fs::write(&thread, "").unwrap();
    let thread_arg = thread.to_str().unwrap();

    let passed = run(
        &home,
        &["verify", "--thread", thread_arg, "--json", "--", "true"],
    );
    assert!(passed.status.success(), "{passed:?}");
    let report: serde_json::Value = serde_json::from_slice(&passed.stdout).unwrap();
    assert_eq!(report["ok"], true);
    assert_eq!(report["command"], "true");

    let failed = run(
        &home,
        &["verify", "--thread", thread_arg, "--json", "--", "false"],
    );
    assert!(!failed.status.success());
    let report: serde_json::Value = serde_json::from_slice(&failed.stdout).unwrap();
    assert_eq!(report["ok"], false);

    let evidence = run(&home, &["evidence", "--thread", thread_arg, "--json"]);
    assert!(evidence.status.success(), "{evidence:?}");
    let report: serde_json::Value = serde_json::from_slice(&evidence.stdout).unwrap();
    assert_eq!(report["verifications"].as_array().unwrap().len(), 2);
    assert_eq!(report["verifications"][0]["command"], "true");
    assert_eq!(report["verifications"][1]["ok"], false);
}

#[test]
fn plan_steps_close_only_after_step_verification() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    let thread = directory.path().join("thread.jsonl");
    std::fs::write(&thread, "").unwrap();
    let thread_arg = thread.to_str().unwrap();

    let added = run(&home, &["plan", "add", "write the tests"]);
    assert!(added.status.success(), "{added:?}");
    assert!(String::from_utf8_lossy(&added.stdout).contains("p-1"));

    let list = run(&home, &["plan", "list", "--json"]);
    let steps: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert_eq!(steps[0]["id"], "p-1");
    assert_eq!(steps[0]["done"], false);

    let refused = run(&home, &["plan", "done", "1", "--thread", thread_arg]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("no passing verification"));

    let verified = run(
        &home,
        &[
            "verify", "--step", "p-1", "--thread", thread_arg, "--", "true",
        ],
    );
    assert!(verified.status.success(), "{verified:?}");

    let closed = run(&home, &["plan", "done", "1", "--thread", thread_arg]);
    assert!(closed.status.success(), "{closed:?}");

    let evidence = run(&home, &["evidence", "--thread", thread_arg, "--json"]);
    let report: serde_json::Value = serde_json::from_slice(&evidence.stdout).unwrap();
    assert_eq!(report["plan"][0]["done"], true);
    assert_eq!(report["verifications"][0]["step"], "p-1");
}

#[test]
fn stats_json_reports_cache_hit_and_tokens_per_turn() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    std::fs::create_dir_all(&home).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    std::fs::write(
        home.join("metrics.jsonl"),
        format!("{now}\tturn\tok\n{now}\ttokens\t1000,200,500\n"),
    )
    .unwrap();
    let output = run(&home, &["stats", "--days", "1", "--json"]);
    assert!(output.status.success(), "{output:?}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["cache_hit_percent"], 50);
    assert_eq!(report["tokens_per_turn"], 1200);
}

#[test]
fn projects_json_filters_and_limits() {
    let directory = tempfile::tempdir().unwrap();
    let niucode = directory.path().join("niucode-home");
    let second = directory.path().join("second");
    std::fs::create_dir_all(&second).unwrap();

    for (key, cwd, last_seen, sessions) in [
        ("demo-key", directory.path(), 100, 0),
        ("second-key", second.as_path(), 200, 2),
    ] {
        let project = niucode.join("projects").join(key);
        std::fs::create_dir_all(project.join("sessions")).unwrap();
        std::fs::write(
            project.join("project.toml"),
            format!(
                "schema_version = 1\ncwd = \"{}\"\nlast_seen = {last_seen}\n",
                cwd.display()
            ),
        )
        .unwrap();
        for index in 0..sessions {
            std::fs::write(
                project.join("sessions").join(format!("s{index}.jsonl")),
                "{}\n",
            )
            .unwrap();
        }
    }

    let home = directory.path().join(".niuse");
    let env = [("NIUCODE_HOME", niucode.to_str().unwrap())];

    // Temporary checkouts are hidden unless --all is passed.
    let filtered = run_with_env(&home, &env, &["projects", "--json"]);
    assert!(filtered.status.success(), "{filtered:?}");
    let filtered: serde_json::Value = serde_json::from_slice(&filtered.stdout).expect("valid JSON");
    assert_eq!(filtered["total"], 0);
    assert_eq!(filtered["shown"], 0);

    let output = run_with_env(
        &home,
        &env,
        &["projects", "--json", "--all", "--limit", "1"],
    );
    assert!(output.status.success(), "{output:?}");
    let registry: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(registry["all"], 2);
    assert_eq!(registry["total"], 2);
    assert_eq!(registry["shown"], 1);
    let project = &registry["projects"][0];
    assert_eq!(project["key"], "second-key");
    assert_eq!(project["sessions"], 2);
    assert_eq!(project["missing"], false);
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

#[test]
fn profile_show_json_reports_missing_files() {
    let directory = tempfile::tempdir().unwrap();
    let home = directory.path().join(".niuse");
    for command in ["user", "identity"] {
        let output = run(&home, &[command, "show", "--json"]);
        assert!(output.status.success(), "{output:?}");
        let profile: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("valid JSON");
        assert!(profile["path"].as_str().is_some());
        assert!(profile["text"].is_null());
    }
}
