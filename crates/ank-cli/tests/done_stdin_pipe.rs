//! `done` finishes when the paths its verifiers read outgrow a pipe
//! (TASK-f2b8aedcd9a4).
//!
//! **Why 4000 paths of 40 bytes.** `done` hands every tracked path its inputs
//! match to `git hash-object --stdin-paths`, 41 bytes in and a 41-byte hash out
//! per path. Written whole before the output was read, the child stopped once
//! its stdout pipe was full and the parent once the stdin pipe filled behind
//! it: measured on Linux, 3193 such paths passed and 3194 hung, against the
//! 2 * 65536 / 41 = 3197 the two 64 KiB pipes predict. 4000 is past that on
//! Linux and further past it where pipes are smaller.
//!
//! **A deadline of the test's own.** A hang is the failure, so the binary is
//! polled and killed past the deadline rather than waited on forever.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "test/done-stdin-pipe";
const PATHS: usize = 4000;
const DEADLINE: Duration = Duration::from_secs(120);

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("done-stdin-pipe-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn command(program: &str, dir: &Path, args: &[&str]) -> Command {
    let config = isolated_git_config();
    let home = scratch::root();
    let mut c = Command::new(program);
    c.args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home)
        .stdin(Stdio::null());
    c
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = command(program, dir, args)
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Runs the binary and kills it past `DEADLINE`; `None` means it hung.
fn run_with_deadline(dir: &Path, args: &[&str]) -> Option<Output> {
    let mut child = command(ANK, dir, args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("ank must run");
    let start = Instant::now();
    while start.elapsed() < DEADLINE {
        if child.try_wait().expect("try_wait").is_some() {
            return Some(child.wait_with_output().expect("output"));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _ = child.kill();
    let _ = child.wait();
    None
}

#[test]
fn done_finishes_when_the_hashed_path_list_outgrows_the_pipe() {
    let root = scratch::dir("done-stdin-pipe");
    let repo = root.join("repo");
    let files = repo.join("f");
    fs::create_dir_all(&files).unwrap();
    for i in 0..PATHS {
        // "f/" and 38 digits: 40 bytes a path, 41 a line.
        fs::write(files.join(format!("{i:038}")), "x").unwrap();
    }
    run("git", &repo, &["init", "-q", "-b", "main"]);
    run(ANK, &repo, &["init"]);
    run(ANK, &repo, &["config", "default_branch", "main"]);
    run(ANK, &repo, &["config", "verifiers.v.run", "git --version"]);
    run(ANK, &repo, &["config", "verifiers.v.inputs", r#"["**"]"#]);
    run("git", &repo, &["add", "-A"]);
    run("git", &repo, &["commit", "-q", "-m", "init"]);
    let tracked = run("git", &repo, &["ls-files"]).lines().count();
    assert!(tracked > PATHS, "{tracked} tracked paths");

    let new = run(
        ANK,
        &repo,
        &[
            "new",
            "task",
            "--title",
            "many inputs",
            "--scope",
            "f/**",
            "--criteria",
            "the verifier passes",
            "--verify",
            "v",
            "--json",
        ],
    );
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    run("git", &repo, &["add", "-A"]);
    run("git", &repo, &["commit", "-q", "-m", "task"]);
    run(ANK, &repo, &["claim", &id]);

    let done = run_with_deadline(&repo, &["done"]).unwrap_or_else(|| {
        panic!("ank done over {tracked} input paths hung past {DEADLINE:?} and was killed")
    });
    assert!(
        done.status.success(),
        "{}{}",
        String::from_utf8_lossy(&done.stdout),
        String::from_utf8_lossy(&done.stderr)
    );
}
