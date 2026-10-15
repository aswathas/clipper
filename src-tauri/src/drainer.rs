//! The queue drainer: batch `inbox.jsonl`, run the provider, file the notes.

use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::{
    clock,
    config::Config,
    prompts,
    queue::{self, QueueEntry},
};

/// What a single drain pass did.
#[derive(Debug, Default)]
pub struct BatchOutcome {
    /// Queue lines taken out of `inbox.jsonl`.
    pub picked: usize,
    /// Notes filed into the vault.
    pub written: Vec<PathBuf>,
    /// Entries moved to `deadletter.jsonl`.
    pub deadlettered: usize,
    /// Why the batch was deadlettered, if it was.
    pub reason: Option<String>,
}

/// A provider invocation's result.
#[derive(Debug)]
struct ProviderRun {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

pub struct Drainer {
    cfg: Config,
    lock: Arc<Mutex<()>>,
    /// `extract.md`, read once at startup.
    header: String,
}

impl Drainer {
    pub fn new(cfg: Config, lock: Arc<Mutex<()>>) -> io::Result<Self> {
        cfg.ensure_dirs()?;
        let header = prompts::ensure_extract_prompt(&cfg.extract_prompt_path())?;
        Ok(Self { cfg, lock, header })
    }

    /// Process at most one batch. Safe to call from tests and from the loop.
    pub fn run_once(&self) -> io::Result<BatchOutcome> {
        let inbox = self.cfg.inbox_jsonl();
        let processing = self.cfg.processing_jsonl();

        // ---- rotate: take the first `batch_size` lines, leave the rest ----
        let batch_lines = {
            let _guard = self.lock.lock().unwrap_or_else(|p| p.into_inner());
            let lines = queue::read_lines(&inbox)?;
            if lines.is_empty() {
                return Ok(BatchOutcome::default());
            }
            let take = self.cfg.queue.batch_size.clamp(1, lines.len());
            let batch: Vec<String> = lines[..take].to_vec();
            let rest: Vec<String> = lines[take..].to_vec();
            queue::write_lines_atomic(&inbox, &rest)?;
            queue::write_lines_atomic(&processing, &batch)?;
            batch
        };

        let mut outcome = BatchOutcome {
            picked: batch_lines.len(),
            ..Default::default()
        };

        // ---- parse; malformed lines are poison, so they go to deadletter ----
        let mut entries: Vec<QueueEntry> = Vec::new();
        let mut bad_json = Vec::new();
        for line in &batch_lines {
            match serde_json::from_str::<QueueEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(_) => bad_json.push(line.clone()),
            }
        }
        for line in &bad_json {
            queue::append_json_line(
                &self.cfg.deadletter_jsonl(),
                &serde_json::json!({ "raw": line, "reason": "bad-json", "at": clock::now_iso8601() }),
            )?;
        }
        outcome.deadlettered += bad_json.len();

        if entries.is_empty() {
            let _ = fs::remove_file(&processing);
            outcome.reason = Some("bad-json".into());
            return Ok(outcome);
        }

        // ---- build the batch prompt ----
        let staging = self.cfg.staging_dir();
        fs::create_dir_all(&staging)?;
        let prompt_path = staging.join(format!("batch_{}.md", uuid::Uuid::new_v4()));
        let targets: Vec<String> = entries.iter().map(QueueEntry::target).collect();
        let prompt_text = build_prompt(&self.header, &targets);
        fs::write(&prompt_path, &prompt_text)?;

        // ---- run the provider ----
        let run = match self.run_provider(&prompt_path, &prompt_text) {
            Ok(run) => run,
            Err(e) => {
                self.deadletter(&entries, &format!("spawn-failed: {e}"))?;
                cleanup(&processing, &prompt_path);
                outcome.deadlettered += entries.len();
                outcome.reason = Some("spawn-failed".into());
                return Ok(outcome);
            }
        };

        if run.timed_out {
            self.deadletter(&entries, "timeout")?;
            cleanup(&processing, &prompt_path);
            outcome.deadlettered += entries.len();
            outcome.reason = Some("timeout".into());
            return Ok(outcome);
        }

        if run.exit_code != Some(0) {
            let reason = format!("exit-{}", run.exit_code.unwrap_or(-1));
            if !run.stderr.trim().is_empty() {
                eprintln!("drainer: provider stderr: {}", run.stderr.trim());
            }
            self.deadletter(&entries, &reason)?;
            cleanup(&processing, &prompt_path);
            outcome.deadlettered += entries.len();
            outcome.reason = Some(reason);
            return Ok(outcome);
        }

        // ---- stdout must be one existing staging .md path per input ----
        let reported: Vec<String> = run
            .stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        let mut sources: Vec<PathBuf> = Vec::with_capacity(reported.len());
        let mut stdout_ok = reported.len() == entries.len();
        if stdout_ok {
            for line in &reported {
                let candidate = if Path::new(line).is_absolute() {
                    PathBuf::from(line)
                } else {
                    staging.join(line)
                };
                if candidate.is_file() {
                    sources.push(candidate);
                } else {
                    stdout_ok = false;
                    break;
                }
            }
        }
        if !stdout_ok {
            self.deadletter(&entries, "bad-stdout")?;
            cleanup(&processing, &prompt_path);
            outcome.deadlettered += entries.len();
            outcome.reason = Some("bad-stdout".into());
            return Ok(outcome);
        }

        // ---- file the notes into the vault, then archive their hashes ----
        let vault_inbox = self.cfg.vault_inbox();
        fs::create_dir_all(&vault_inbox)?;
        let stamp = clock::file_stamp();

        for (entry, source) in entries.iter().zip(sources.iter()) {
            // Read before moving: an unreadable note is a bad batch.
            fs::read(source)?;
            let dest = unique_destination(&vault_inbox, &stamp, &entry.source, &entry.id);
            if fs::rename(source, &dest).is_err() {
                fs::copy(source, &dest)?;
                let _ = fs::remove_file(source);
            }
            queue::append_json_line(
                &self.cfg.archive_jsonl(),
                &serde_json::json!({
                    "id": entry.id,
                    "hash": queue::url_hash(&entry.target()),
                    "at": clock::now_iso8601(),
                }),
            )?;
            outcome.written.push(dest);
        }

        cleanup(&processing, &prompt_path);
        Ok(outcome)
    }

    /// Poll `inbox.jsonl`, and also wake on file change (debounced).
    pub async fn run_forever(self: Arc<Self>) {
        let poll_ms = self.cfg.queue.poll_interval_ms.max(250);
        let watch_dir = self.cfg.queue_dir();
        let _ = fs::create_dir_all(&watch_dir);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
            if event.is_ok() {
                let _ = tx.send(());
            }
        })
        .ok();
        if let Some(watcher) = watcher.as_mut() {
            use notify::Watcher;
            let _ = watcher.watch(&watch_dir, notify::RecursiveMode::NonRecursive);
        }

        let mut ticker = tokio::time::interval(Duration::from_millis(poll_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = ticker.tick() => {}
                Some(_) = rx.recv() => {
                    // Debounce: let the writer finish, then swallow the burst.
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    while rx.try_recv().is_ok() {}
                }
            }

            let me = Arc::clone(&self);
            let _ = tokio::task::spawn_blocking(move || match me.run_once() {
                Ok(outcome) if outcome.picked > 0 => println!(
                    "drainer: picked={} written={} deadletter={}{}",
                    outcome.picked,
                    outcome.written.len(),
                    outcome.deadlettered,
                    outcome
                        .reason
                        .as_deref()
                        .map(|r| format!(" reason={r}"))
                        .unwrap_or_default()
                ),
                Ok(_) => {}
                Err(e) => eprintln!("drainer: {e}"),
            })
            .await;
        }
    }

    /// Append every entry of a failed batch to `deadletter.jsonl`.
    fn deadletter(&self, entries: &[QueueEntry], reason: &str) -> io::Result<()> {
        for entry in entries {
            queue::append_json_line(
                &self.cfg.deadletter_jsonl(),
                &serde_json::json!({
                    "id": entry.id,
                    "url": entry.target(),
                    "source": entry.source,
                    "reason": reason,
                    "at": clock::now_iso8601(),
                }),
            )?;
        }
        Ok(())
    }

    /// Spawn the configured provider, pipe the prompt in, and wait with a timeout.
    fn run_provider(&self, prompt_file: &Path, prompt_text: &str) -> io::Result<ProviderRun> {
        let provider = &self.cfg.provider.claude;
        let pipe_prompt = provider.stdin == "prompt-file";

        let mut command = Command::new(&provider.command);
        command.args(&provider.args);
        if !pipe_prompt {
            command.arg(prompt_file);
        }
        command
            .stdin(if pipe_prompt { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn()?;

        if pipe_prompt {
            if let Some(mut stdin) = child.stdin.take() {
                let payload = prompt_text.as_bytes().to_vec();
                // Feed stdin off-thread so a large prompt cannot deadlock the wait loop.
                thread::spawn(move || {
                    let _ = stdin.write_all(&payload);
                    let _ = stdin.flush();
                });
            }
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();
        let stdout_reader = thread::spawn(move || drain(stdout));
        let stderr_reader = thread::spawn(move || drain(stderr));

        let deadline = Instant::now() + Duration::from_secs(self.cfg.queue.timeout_s.max(1));
        let mut timed_out = false;
        let exit = loop {
            if let Some(status) = child.try_wait()? {
                break Some(status);
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                timed_out = true;
                break child.wait().ok();
            }
            thread::sleep(Duration::from_millis(50));
        };

        Ok(ProviderRun {
            exit_code: exit.and_then(|status| status.code()),
            stdout: stdout_reader.join().unwrap_or_default(),
            stderr: stderr_reader.join().unwrap_or_default(),
            timed_out,
        })
    }
}

/// Read a child pipe to end, lossily.
fn drain(pipe: Option<impl Read>) -> String {
    let mut buffer = Vec::new();
    if let Some(mut pipe) = pipe {
        let _ = pipe.read_to_end(&mut buffer);
    }
    String::from_utf8_lossy(&buffer).into_owned()
}

/// `extract.md` header + the batch's targets as a JSON list.
fn build_prompt(header: &str, targets: &[String]) -> String {
    let list = serde_json::to_string_pretty(targets).unwrap_or_else(|_| "[]".to_string());
    format!("{header}\n\nURLs (JSON list, in order):\n\n{list}\n")
}

/// `<stamp>_<source>_<id>.md`, with `-<n>` appended on collision.
fn unique_destination(dir: &Path, stamp: &str, source: &str, id: &str) -> PathBuf {
    let base = format!("{stamp}_{source}_{id}");
    let mut candidate = dir.join(format!("{base}.md"));
    let mut suffix = 1;
    while candidate.exists() {
        candidate = dir.join(format!("{base}-{suffix}.md"));
        suffix += 1;
    }
    candidate
}

fn cleanup(processing: &Path, prompt: &Path) {
    let _ = fs::remove_file(processing);
    let _ = fs::remove_file(prompt);
}
