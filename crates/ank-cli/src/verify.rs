//! Execution of named verifiers through `sh -c` (§4).
//!
//! **Ank runs the verification itself.** That is the whole point: if the agent
//! ran the tests and then reported the result through `--proof`, nothing would
//! be anchored — it could simply claim it passes. The task declares its
//! verifiers, `done` runs them, and the agent never self-reports.
//!
//! A verifier is **named**, declared in `config.yml`, never an inline command.
//! A task can arrive through a pull request from a fork, and an inline command
//! would be arbitrary code execution triggered by `ank done`. Git has exactly
//! this problem with hooks and solved it by never running them on clone; here
//! the verifiers live in a file the repository controls, so changing one goes
//! through review like any other change.
//!
//! **A broken environment is not a task failure.** `sh` missing, a command that
//! does not exist, a process that will not spawn: all code 9 with the exact
//! command to run. Code 5 stays reserved for a verifier that ran and said no —
//! confusing the two sends an agent to fix code that was never wrong.
//!
//! Dispatch routes to no verifier: `done` calls this module.

use crate::cli::CliError;
use crate::config::Verifier;
use ank_contract::ExitCode;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, CliError>;

const GIT_FOR_WINDOWS: &str = "https://git-scm.com/download/win";
const POSIX_SH: &str = "install a POSIX sh, or Git for Windows which ships one";

/// What a verifier did. `ok` is the only thing that decides the transition;
/// the rest is what gets anchored in the proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub name: String,
    pub ok: bool,
    pub code: Option<i32>,
    pub elapsed: Duration,
    /// Hash of what the verifier printed, stdout and stderr together. The
    /// proof anchors this rather than the text: a proof entry is a one-line
    /// diff, and megabytes of test output in a task file would make the format
    /// unreadable for the one thing it exists to record.
    pub output_hash: String,
    pub timed_out: bool,
}

impl Outcome {
    /// `local/<output hash>@<head sha>` — the reference form of §4's trust
    /// hierarchy for a proof Ank produced itself.
    pub fn reference(&self, head: &str, dirty: bool) -> String {
        let head: String = head.chars().take(7).collect();
        // The dirty marker is not decoration: an agent's nominal case is an
        // uncommitted tree, so a proof naming only the HEAD sha would almost
        // always point at a state that was never tested.
        let suffix = if dirty { "+dirty" } else { "" };
        format!("local/{}@{head}{suffix}", &self.output_hash[..12])
    }
}

/// Hash of the definition that actually ran, `<name>@<hash>`.
///
/// `config.yml` stays editable by an agent, and replacing a verifier with
/// `true` is the obvious workaround. This is the first of the two defences,
/// and neither rests on good faith: what ran is anchored in the proof rather
/// than in the current state of the file, so a verifier weakened before or
/// after the `done` — same commit or another — is detectable by comparing this
/// hash with the definition at that commit. The second defence is `check`,
/// which reports the pattern. Faking has to cost more than doing.
pub fn definition_hash(v: &Verifier) -> String {
    let mut h = Sha256::new();
    h.update(v.run.trim().as_bytes());
    h.update([0]);
    h.update(v.timeout.as_secs().to_string().as_bytes());
    // `inputs` and `env` are appended only when declared (ADR-d8a328e311b9),
    // so a verifier declaring neither keeps the hash every proof recorded
    // before they existed. Each string goes in length-prefixed: two fields
    // can never trade bytes and produce one hash.
    if let Some(inputs) = &v.inputs {
        // A set, not a sequence: the order globs are listed in changes
        // nothing about which files can change the result.
        let globs: std::collections::BTreeSet<&str> = inputs.iter().map(String::as_str).collect();
        h.update(b"\0inputs");
        for g in globs {
            put(&mut h, g);
        }
    }
    if let Some(env) = &v.env {
        h.update(b"\0env");
        for (field, value) in [
            ("image", env.image.as_deref()),
            ("platform", env.platform.as_deref()),
            // Resolved by the parser: `network: none` written out and left to
            // its default under an image are one environment.
            ("network", env.network.map(crate::config::Network::as_str)),
        ] {
            if let Some(value) = value {
                put(&mut h, field);
                put(&mut h, value);
            }
        }
        for (name, value) in &env.vars {
            put(&mut h, "var");
            put(&mut h, name);
            put(&mut h, value);
        }
    }
    hex::encode(h.finalize())[..12].to_string()
}

fn put(h: &mut Sha256, s: &str) {
    h.update((s.len() as u64).to_le_bytes());
    h.update(s.as_bytes());
}

pub fn definition_ref(name: &str, v: &Verifier) -> String {
    format!("{name}@{}", definition_hash(v))
}

/// Locates a POSIX shell.
///
/// `sh` on `PATH` when there is one. Otherwise, on Windows, the one Git for
/// Windows ships: git is already a hard dependency, so the shell comes free and
/// verifiers stay written once, in POSIX syntax, for the whole team. Deriving
/// it from git's own location on `PATH` keeps this out of `git.rs` and out of
/// the plumbing list — asking git where it lives would be a new subcommand for
/// something a directory walk answers.
///
/// **Never a silent fallback to `cmd`.** A verifier written in POSIX syntax and
/// handed to `cmd` fails in ways that look like the code is wrong.
pub fn find_sh() -> Result<PathBuf> {
    if let Some(found) = on_path("sh") {
        return Ok(found);
    }
    if let Some(git) = on_path("git") {
        // C:\Program Files\Git\cmd\git.exe -> C:\Program Files\Git\bin\sh.exe
        let mut dir = git.parent();
        for _ in 0..3 {
            let Some(base) = dir else { break };
            for candidate in ["bin/sh.exe", "usr/bin/sh.exe", "bin/sh"] {
                let p = base.join(candidate);
                if p.is_file() {
                    return Ok(p);
                }
            }
            dir = base.parent();
        }
    }
    Err(CliError::new(
        ExitCode::Environment,
        "sh not found: verifiers run through sh -c on the three platforms",
    )
    .with_hint(if cfg!(windows) {
        GIT_FOR_WINDOWS
    } else {
        POSIX_SH
    }))
}

fn on_path(name: &str) -> Option<PathBuf> {
    let exts: &[&str] = if cfg!(windows) { &["", ".exe"] } else { &[""] };
    let path = std::env::var_os(ank_contract::env::PATH)?;
    for dir in std::env::split_paths(&path) {
        for ext in exts {
            let p = dir.join(format!("{name}{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Runs one verifier in `cwd`.
///
/// Output goes to a temporary file rather than a pipe, deliberately: a verifier
/// that prints more than the pipe buffer holds would deadlock against a parent
/// that is waiting for it to exit, and a test suite is exactly the kind of
/// command that prints a lot.
pub fn run(cwd: &Path, name: &str, v: &Verifier) -> Result<Outcome> {
    Ok(run_until(cwd, name, v, &|| false)?.expect("a run nothing stops is never cancelled"))
}

/// [`run`], polling `stop` while the verifier runs: once it answers true the
/// verifier is killed and `None` comes back, an outcome nobody will read.
///
/// This is what lets `done` run verifiers side by side (TASK-26eab418555a):
/// once one fails, the ones declared after it can no longer change the
/// answer, and letting them run on would leave them running after `done`
/// has already said no.
pub fn run_until(
    cwd: &Path,
    name: &str,
    v: &Verifier,
    stop: &dyn Fn() -> bool,
) -> Result<Option<Outcome>> {
    let sh = find_sh()?;
    let out_path = std::env::temp_dir().join(format!(
        "ank-verify-{}-{}.out",
        std::process::id(),
        name.replace(|c: char| !c.is_alphanumeric(), "-")
    ));
    let file = std::fs::File::create(&out_path).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot capture verifier output: {e}"),
        )
    })?;
    let errs = file.try_clone().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot capture verifier output: {e}"),
        )
    })?;

    let started = Instant::now();
    let mut command = Command::new(&sh);
    // A group of its own, so that killing the verifier kills what it started:
    // `sh -c "a; b"` keeps the shell as a parent, and a kill aimed at the
    // shell alone leaves `b` running with nobody waiting for it.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    let mut child = command
        .current_dir(cwd)
        .arg("-c")
        .arg(&v.run)
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(errs))
        .spawn()
        .map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("cannot run verifier '{name}': {e}"),
            )
            .with_hint(format!("{} -c {:?}", sh.display(), v.run))
        })?;

    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if stop() {
                    kill_tree(&mut child);
                    let _ = std::fs::remove_file(&out_path);
                    return Ok(None);
                }
                if started.elapsed() >= v.timeout {
                    kill_tree(&mut child);
                    timed_out = true;
                    break std::process::ExitStatus::default();
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                return Err(CliError::new(
                    ExitCode::Environment,
                    format!("cannot wait for verifier '{name}': {e}"),
                ))
            }
        }
    };

    let elapsed = started.elapsed();
    let output = std::fs::read(&out_path).unwrap_or_default();
    let _ = std::fs::remove_file(&out_path);
    let code = status.code();

    // 126 and 127 are the shell's own way of saying "I could not run that":
    // not executable, and not found. Neither is the agent's code failing, so
    // neither is a code 5.
    if !timed_out {
        if let Some(c @ (126 | 127)) = code {
            let tail = String::from_utf8_lossy(&output);
            let tail = tail.lines().last().unwrap_or("").trim();
            return Err(CliError::new(
                ExitCode::Environment,
                format!("verifier '{name}' could not run (shell code {c}): {tail}"),
            )
            .with_hint(format!("sh -c {:?}", v.run)));
        }
    }

    let mut h = Sha256::new();
    h.update(&output);
    Ok(Some(Outcome {
        name: name.to_string(),
        ok: !timed_out && status.success(),
        code,
        elapsed,
        output_hash: hex::encode(h.finalize()),
        timed_out,
    }))
}

/// Kills a verifier and everything it started, then reaps it.
///
/// On Unix the verifier leads a process group of its own and the group is
/// killed whole. On Windows `taskkill /T` walks the tree from the shell down.
/// Both through a process rather than a binding: no crate enters for one
/// signal, and the shell's own kill is the fallback when neither runs.
fn kill_tree(child: &mut std::process::Child) {
    let pid = child.id().to_string();
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(not(any(unix, windows)))]
    let _ = pid;
    let _ = child.kill();
    let _ = child.wait();
}

/// The code 5 a failed verifier deserves, with the elapsed time when it ran out
/// of it.
pub fn failure(outcome: &Outcome, run: &str) -> CliError {
    let detail = if outcome.timed_out {
        format!(
            "verifier '{}' timed out after {:.1}s",
            outcome.name,
            outcome.elapsed.as_secs_f64()
        )
    } else {
        format!(
            "verifier '{}' failed (exit {})",
            outcome.name,
            outcome
                .code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        )
    };
    CliError::new(ExitCode::Proof, detail).with_hint(format!("sh -c {run:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verifier(run: &str, secs: u64) -> Verifier {
        Verifier {
            run: run.to_string(),
            timeout: Duration::from_secs(secs),
            inputs: None,
            env: None,
        }
    }

    fn cwd() -> PathBuf {
        std::env::temp_dir()
    }

    #[test]
    fn sh_is_found_on_the_three_platforms() {
        // git is a hard dependency and ships sh on Windows, so this must
        // succeed everywhere the tool is supported. On this machine sh is not
        // on PATH at all, which is precisely the case the fallback exists for.
        let sh = find_sh().expect("sh must be resolvable wherever ank runs");
        assert!(sh.is_file(), "{}", sh.display());
    }

    #[test]
    fn a_verifier_that_passes_is_ok_and_hashes_its_output() {
        let out = run(&cwd(), "greet", &verifier("echo hello", 30)).unwrap();
        assert!(out.ok);
        assert_eq!(out.code, Some(0));
        assert!(!out.timed_out);
        assert_eq!(out.output_hash.len(), 64);

        // The hash is of the output, so two different outputs differ.
        let other = run(&cwd(), "greet", &verifier("echo goodbye", 30)).unwrap();
        assert_ne!(out.output_hash, other.output_hash);
        // And the same output hashes the same, which is what makes a proof
        // comparable across runs.
        let again = run(&cwd(), "greet", &verifier("echo hello", 30)).unwrap();
        assert_eq!(out.output_hash, again.output_hash);
    }

    #[test]
    fn a_verifier_that_says_no_is_a_failure_and_not_an_environment_problem() {
        let out = run(&cwd(), "nope", &verifier("exit 1", 30)).unwrap();
        assert!(!out.ok);
        assert_eq!(out.code, Some(1));

        let err = failure(&out, "exit 1");
        assert_eq!(
            err.code,
            ExitCode::Proof,
            "a verifier that ran and refused is code 5"
        );
        assert!(err.message.contains("nope"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    #[test]
    fn a_command_that_does_not_exist_is_the_environment_and_not_the_code() {
        // 127 from the shell. Reporting this as a failed verifier would send
        // an agent to fix code that was never wrong.
        let err = run(&cwd(), "missing", &verifier("no-such-command-anywhere", 30)).unwrap_err();
        assert_eq!(err.code, ExitCode::Environment, "{}", err.message);
        assert!(err.message.contains("missing"), "{}", err.message);
        assert!(err.hint.is_some(), "and it says what to run");
    }

    #[test]
    fn a_timeout_is_a_failure_carrying_the_elapsed_time() {
        let out = run(&cwd(), "slow", &verifier("sleep 30", 1)).unwrap();
        assert!(out.timed_out);
        assert!(!out.ok);
        assert!(out.elapsed >= Duration::from_secs(1));
        assert!(
            out.elapsed < Duration::from_secs(20),
            "the child is killed, not waited on: {:?}",
            out.elapsed
        );

        let err = failure(&out, "sleep 30");
        assert_eq!(
            err.code,
            ExitCode::Proof,
            "exceeding the timeout is a code 5 (§4)"
        );
        assert!(err.message.contains("timed out"), "{}", err.message);
        assert!(err.message.contains('s'), "with the elapsed time");
    }

    #[test]
    fn a_verifier_that_prints_a_great_deal_does_not_deadlock() {
        // The reason output goes to a file rather than a pipe: a test suite is
        // exactly the kind of command that outruns a pipe buffer, and the
        // deadlock would look like a hung tool.
        let out = run(
            &cwd(),
            "chatty",
            &verifier(
                "i=0; while [ $i -lt 4000 ]; do echo 'a line of output'; i=$((i+1)); done",
                60,
            ),
        )
        .unwrap();
        assert!(out.ok, "code {:?}", out.code);
    }

    #[test]
    fn the_definition_hash_moves_when_the_definition_does() {
        let a = verifier("cargo test --workspace", 600);
        let same = verifier("  cargo test --workspace  ", 600);
        let weakened = verifier("true", 600);
        let slower = verifier("cargo test --workspace", 60);

        assert_eq!(definition_hash(&a), definition_hash(&same), "trimmed alike");
        assert_ne!(
            definition_hash(&a),
            definition_hash(&weakened),
            "replacing a verifier with true is the workaround this anchors"
        );
        assert_ne!(
            definition_hash(&a),
            definition_hash(&slower),
            "the timeout is part of what ran"
        );
        assert_eq!(
            definition_ref("cargo-test", &a),
            format!("cargo-test@{}", definition_hash(&a))
        );
    }

    /// ADR-d8a328e311b9: `inputs` and `env` are part of what ran, and a
    /// verifier declaring neither hashes exactly as it did before they existed.
    #[test]
    fn the_definition_hash_covers_inputs_and_env_and_nothing_when_neither_is_declared() {
        let p = std::path::Path::new("config.yml");
        let hash = |yaml: &str| {
            let cfg = crate::config::parse(&format!("schema: 1\nverifiers:\n  t:\n{yaml}"), p)
                .unwrap_or_else(|e| panic!("{yaml}: {}", e.message));
            definition_hash(&cfg.verifiers["t"])
        };
        let bare = hash("    run: cargo test\n");
        // sha256("cargo test\0600"), first twelve hex digits: the formula
        // every proof recorded so far was hashed with.
        assert_eq!(bare, "ea889101030e");
        assert_eq!(
            hash("    run: cargo test\n    env: {}\n"),
            bare,
            "an env declaring nothing is no environment"
        );

        let inputs = hash("    run: cargo test\n    inputs: [\"src/**\", \"Cargo.toml\"]\n");
        assert_ne!(inputs, bare);
        assert_eq!(
            hash("    run: cargo test\n    inputs:\n      - Cargo.toml\n      - src/**\n"),
            inputs,
            "a set of globs, whatever order and style lists them"
        );
        assert_ne!(
            hash("    run: cargo test\n    inputs: [\"src/**\"]\n"),
            inputs
        );

        let image = "    run: cargo test\n    env:\n      image: r@sha256:\
                     0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n      \
                     platform: linux/amd64\n";
        let pinned = hash(image);
        assert_ne!(pinned, bare);
        assert_eq!(
            hash(&format!("{image}      network: none\n")),
            pinned,
            "network none written out is the default under an image"
        );
        assert_ne!(hash(&format!("{image}      network: open\n")), pinned);
        let with_var = hash(&format!("{image}      vars:\n        LANG: C\n"));
        assert_ne!(with_var, pinned);
        assert_ne!(
            hash(&format!("{image}      vars:\n        LANG: POSIX\n")),
            with_var
        );
        assert_ne!(
            hash("    run: cargo test\n    env:\n      platform: linux/arm64\n"),
            hash("    run: cargo test\n    env:\n      platform: linux/amd64\n")
        );
    }

    #[test]
    fn the_local_reference_names_the_head_and_says_when_the_tree_was_dirty() {
        let out = Outcome {
            name: "t".into(),
            ok: true,
            code: Some(0),
            elapsed: Duration::from_secs(1),
            output_hash: "0123456789abcdef".repeat(4),
            timed_out: false,
        };
        assert_eq!(
            out.reference("a3f9c21deadbeef", false),
            "local/0123456789ab@a3f9c21"
        );
        assert_eq!(
            out.reference("a3f9c21deadbeef", true),
            "local/0123456789ab@a3f9c21+dirty",
            "an uncommitted tree is the agent's nominal case, and must show"
        );
    }
}
