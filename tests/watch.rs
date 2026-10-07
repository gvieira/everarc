use std::{
    env, fs,
    path::PathBuf,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

struct WatchTest {
    directory: PathBuf,
    child: Option<Child>,
}

impl WatchTest {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after the Unix epoch")
            .as_nanos();
        let directory =
            env::temp_dir().join(format!("everarc-watch-{}-{unique}", std::process::id()));
        fs::create_dir(&directory).expect("temporary directory creates");
        let mut test = Self {
            directory,
            child: None,
        };
        // Resolve macOS's /var -> /private/var alias so watcher event paths
        // and the CLI's absolute configuration path use the same spelling.
        test.directory = fs::canonicalize(&test.directory).expect("temporary directory resolves");
        test
    }

    fn write_config(&self, note: &str, contribution: &str) {
        let config = format!(
            r#"
[display]
locale = "en-US"

[plan]
currency = "USD"
start = "2026-01"
end = "2026-01"

[actual_months."2026-01"]
note = "{note}"

[[scenarios]]
id = "baseline"
name = "Baseline"
annual_inflation = "0"

[[scenarios.assets]]
id = "cash"
name = "Cash"
currency = "USD"
annual_expected_return = "0"
monthly_contribution = {{ amount = "{contribution}", currency = "USD" }}

[[scenarios.assets.holdings]]
id = "opening"
name = "Opening balance"
currency = "USD"
value = "1000"
"#
        );
        fs::write(self.directory.join("everarc.toml"), config).expect("configuration writes");
    }

    fn start(&mut self) {
        let log = fs::File::create(self.directory.join("stderr.log")).expect("stderr log creates");
        self.child = Some(
            Command::new(env!("CARGO_BIN_EXE_everarc"))
                .current_dir(&self.directory)
                .args([
                    "--config",
                    "everarc.toml",
                    "build",
                    "--watch",
                    "--output",
                    "everarc.html",
                    "--data-output",
                    "everarc.json",
                ])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(log)
                .spawn()
                .expect("watch command starts"),
        );
    }

    fn wait_for_outputs(&mut self, note: &str, balance: &str) {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            let status = self
                .child
                .as_mut()
                .expect("watch command has started")
                .try_wait()
                .expect("watch command status reads");
            assert!(
                status.is_none(),
                "watch command exited: {status:?}\n{}",
                self.log()
            );

            // Writes are not atomic: retry missing, partial, or stale outputs.
            let html_matches = fs::read_to_string(self.directory.join("everarc.html"))
                .is_ok_and(|html| html.contains(note) && html.trim_end().ends_with("</html>"));
            let json_matches = fs::read_to_string(self.directory.join("everarc.json"))
                .ok()
                .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
                .is_some_and(|json| {
                    let month = &json["scenarios"][0]["months"][0];
                    month["actual_note"] == note && month["total_balance"] == balance
                });
            if html_matches && json_matches {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for HTML/JSON with note {note:?} and balance {balance}\n{}",
                self.log()
            );
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn log(&self) -> String {
        fs::read_to_string(self.directory.join("stderr.log")).unwrap_or_default()
    }
}

impl Drop for WatchTest {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&self.directory);
    }
}

#[test]
fn watch_rebuilds_html_and_json_after_a_configuration_edit() {
    let mut test = WatchTest::new();
    test.write_config("Initial watch test note", "100");
    test.start();
    // Initial output also confirms the directory watcher is registered.
    test.wait_for_outputs("Initial watch test note", "1100");

    test.write_config("Updated watch test note", "200");
    test.wait_for_outputs("Updated watch test note", "1200");
}
