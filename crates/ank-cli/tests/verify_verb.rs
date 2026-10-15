//! `ank verify` runs the verifiers a change touches and records their runs for
//! the `done` that follows (ADR-c621cb9c6739, TASK-15b59a393a38).
//!
//! **Through the binary, and counted.** Every verifier here appends one line to
//! a counter file outside the repository each time a process runs it, so "the
//! process of that verifier only" is two line counts that did not move and one
//! that did, never a duration that looked short.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "test/verify-verb";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("verify-verb-gitconfig");
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

fn spawn(program: &str, dir: &Path, args: &[&str]) -> Output {
    let config = isolated_git_config();
    let home = scratch::root();
    Command::new(program)
        .args(args)
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
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"))
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir, args);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// `<root>/repo` on `main`, with three verifiers: `a` reads `a/**`, `b` reads
/// `b/**`, and `whole` declares no inputs. The counters live in `<root>`,
/// outside every work tree, so no run moves the content another is keyed on.
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let root = scratch::dir(what);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("a")).unwrap();
        fs::create_dir_all(repo.join("b")).unwrap();
        fs::write(repo.join("a/x"), "a\n").unwrap();
        fs::write(repo.join("b/x"), "b\n").unwrap();
        run("git", &repo, &["init", "-q", "-b", "main"]);
        run(ANK, &repo, &["init"]);
        run(ANK, &repo, &["config", "default_branch", "main"]);
        for (key, value) in [
            ("verifiers.a.run", "echo a >> ../a.count"),
            ("verifiers.a.inputs", r#"["a/**"]"#),
            ("verifiers.b.run", "echo b >> ../b.count"),
            ("verifiers.b.inputs", r#"["b/**"]"#),
            ("verifiers.whole.run", "echo whole >> ../whole.count"),
        ] {
            run(ANK, &repo, &["config", key, value]);
        }
        commit(&repo, "init");
        Fixture { root, repo }
    }

    /// How many processes ran verifier `name`, counted.
    fn count(&self, name: &str) -> usize {
        fs::read_to_string(self.root.join(format!("{name}.count")))
            .map(|t| t.lines().count())
            .unwrap_or(0)
    }

    fn counts(&self) -> (usize, usize, usize) {
        (self.count("a"), self.count("b"), self.count("whole"))
    }

    fn verify(&self) -> String {
        run(ANK, &self.repo, &["verify"])
    }
}

fn commit(dir: &Path, message: &str) {
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", message]);
}

#[test]
fn a_change_under_one_verifier_starts_that_verifier_only_and_one_without_inputs_always() {
    let f = Fixture::new("verify-select");

    // Nothing changed: only the verifier reading the whole tree is touched.
    f.verify();
    assert_eq!(f.counts(), (0, 0, 1), "no inputs means always run");

    fs::write(f.repo.join("a/x"), "a, edited\n").unwrap();
    let out = f.verify();
    assert_eq!(f.counts(), (1, 0, 2), "a's inputs changed, b's did not");
    assert!(out.contains("b  skipped"), "{out}");

    // Untracked and not ignored selects like a modification.
    fs::write(f.repo.join("b/new"), "new\n").unwrap();
    f.verify();
    assert_eq!(f.counts(), (2, 1, 3));

    // A deletion is a modification of a tracked file.
    commit(&f.repo, "both");
    f.verify();
    assert_eq!(
        f.counts(),
        (2, 1, 4),
        "a clean tree on main touches neither"
    );
    fs::remove_file(f.repo.join("b/x")).unwrap();
    f.verify();
    assert_eq!(f.counts(), (2, 2, 5));
}

#[test]
fn a_change_committed_on_the_branch_selects_like_a_modified_file() {
    let f = Fixture::new("verify-branch");
    run("git", &f.repo, &["checkout", "-q", "-b", "feature"]);
    fs::write(f.repo.join("b/x"), "b, on the branch\n").unwrap();
    commit(&f.repo, "b on the branch");

    // The tree is clean against HEAD, and b still runs: what the branch
    // brings to main is a change whether or not it is committed.
    let json = run(ANK, &f.repo, &["verify", "--json"]);
    assert_eq!(f.counts(), (0, 1, 1), "{json}");
    assert!(json.contains(r#""changed":1"#), "{json}");
    assert!(json.contains(r#""skipped":["a"]"#), "{json}");
    let base = run("git", &f.repo, &["rev-parse", "main"]);
    assert!(json.contains(&format!(r#""base":"{base}""#)), "{json}");
}

#[test]
fn verify_needs_no_claim_and_writes_no_proof_no_ref_and_no_entity() {
    let f = Fixture::new("verify-writes");
    fs::write(f.repo.join("a/x"), "a, edited\n").unwrap();
    let snapshot = || {
        (
            run(
                "git",
                &f.repo,
                &[
                    "status",
                    "--porcelain",
                    "--untracked-files=all",
                    "--",
                    ".ank",
                ],
            ),
            run(
                "git",
                &f.repo,
                &["for-each-ref", "--format=%(refname) %(objectname)", "refs/"],
            ),
        )
    };
    let before = snapshot();
    let out = spawn(ANK, &f.repo, &["verify"]);
    assert!(
        out.status.success(),
        "no claim is held, and verify runs: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(f.counts(), (1, 0, 1));
    assert_eq!(snapshot(), before, "no file under .ank/ and no ref moved");
    let runs = fs::read_dir(f.repo.join(".git/ank/runs"))
        .expect("the runs are recorded under the git common directory")
        .count();
    assert_eq!(runs, 2, "one entry per verifier that ran");
}

#[test]
fn a_failing_verifier_is_a_code_five_and_its_run_is_recorded() {
    let f = Fixture::new("verify-fails");
    run(
        ANK,
        &f.repo,
        &["config", "verifiers.a.run", "echo a >> ../a.count; exit 1"],
    );
    fs::write(f.repo.join("a/x"), "a, edited\n").unwrap();
    let out = spawn(ANK, &f.repo, &["verify", "--json"]);
    assert_eq!(out.status.code(), Some(5));
    assert!(out.stdout.is_empty(), "a refusal leaves stdout empty");
    assert_eq!(f.count("a"), 1);
}

#[test]
fn a_following_done_on_the_same_tree_reuses_the_run() {
    let f = Fixture::new("verify-then-done");
    let new = run(
        ANK,
        &f.repo,
        &[
            "new",
            "task",
            "--title",
            "touches a",
            "--scope",
            "a/**",
            "--criteria",
            "the verifiers pass",
            "--verify",
            "a",
            "--verify",
            "whole",
            "--json",
        ],
    );
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    commit(&f.repo, "task");
    run(ANK, &f.repo, &["claim", &id]);

    fs::write(f.repo.join("a/x"), "a, edited\n").unwrap();
    f.verify();
    assert_eq!(f.counts(), (1, 0, 1));

    let done = spawn(ANK, &f.repo, &["done"]);
    assert!(
        done.status.success(),
        "{}",
        String::from_utf8_lossy(&done.stderr)
    );
    assert_eq!(f.counts(), (1, 0, 1), "done started no verifier process");
    let shown = run(ANK, &f.repo, &["show", &id]);
    assert_eq!(
        shown.matches("reused: local").count(),
        2,
        "both entries were reused from verify's runs:\n{shown}"
    );
    assert!(shown.contains(&format!("by: {AGENT}")), "{shown}");
}

#[test]
fn the_verb_is_in_help_and_in_the_machine_surface() {
    let f = Fixture::new("verify-help");
    let help = run(ANK, &f.repo, &["help"]);
    assert!(help.contains("verify"), "{help}");
    let page = run(ANK, &f.repo, &["help", "verify"]);
    assert!(page.contains("merge-base"), "{page}");
    let json = run(ANK, &f.repo, &["help", "--json"]);
    assert!(json.contains(r#""name":"verify""#), "{json}");
}
