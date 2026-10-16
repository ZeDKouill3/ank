//! `done` reuses a passing verifier run whose content key is unchanged
//! (ADR-a2faf59f6ce0, ADR-7224dea611c1, TASK-413bcf394090).
//!
//! **Through the binary, and counted.** Every verifier here appends one line to
//! a counter file outside the repository each time a process runs it, so "no
//! process started" is a line count that did not move, never a duration that
//! looked short.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "test/verifier-reuse";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("verifier-reuse-gitconfig");
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

fn spawn_as(agent: &str, program: &str, dir: &Path, args: &[&str]) -> Output {
    let config = isolated_git_config();
    let home = scratch::root();
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", agent)
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

fn run_as(agent: &str, program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn_as(agent, program, dir, args);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    run_as(AGENT, program, dir, args)
}

/// A fixture: `<root>/repo` is the repository, and the counters live in
/// `<root>`, outside every work tree, so a sibling worktree at `<root>/wt`
/// reaches the same ones as `../<name>.count`.
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    /// Two verifiers with disjoint `inputs`: `a` reads `a/**`, `b` reads
    /// `b/**` and fails while `<root>/b.fail` exists, a file outside its
    /// inputs, so a failure can be arranged without moving its key.
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
            (
                "verifiers.b.run",
                "echo b >> ../b.count && test ! -f ../b.fail",
            ),
            ("verifiers.b.inputs", r#"["b/**"]"#),
        ] {
            run(ANK, &repo, &["config", key, value]);
        }
        run("git", &repo, &["add", "-A"]);
        run("git", &repo, &["commit", "-q", "-m", "init"]);
        Fixture { root, repo }
    }

    /// How many processes ran verifier `name`, counted.
    fn count(&self, name: &str) -> usize {
        fs::read_to_string(self.root.join(format!("{name}.count")))
            .map(|t| t.lines().count())
            .unwrap_or(0)
    }

    fn registry(&self) -> PathBuf {
        let common = run("git", &self.repo, &["rev-parse", "--git-common-dir"]);
        let common = Path::new(&common);
        let common = if common.is_absolute() {
            common.to_path_buf()
        } else {
            self.repo.join(common)
        };
        common.join("ank").join("runs")
    }
}

/// A task over `a` and `b` declaring `verify`, committed and claimed in `dir`.
fn task(dir: &Path, agent: &str, title: &str, verify: &[&str]) -> String {
    let mut args = vec![
        "new",
        "task",
        "--title",
        title,
        "--scope",
        "a/**",
        "--criteria",
        "the verifiers pass",
        "--json",
    ];
    for v in verify {
        args.extend(["--verify", v]);
    }
    let new = run_as(agent, ANK, dir, &args);
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    run_as(agent, "git", dir, &["add", "-A"]);
    run_as(agent, "git", dir, &["commit", "-q", "-m", title]);
    run_as(agent, ANK, dir, &["claim", &id]);
    id
}

fn commit(dir: &Path, message: &str) {
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", message]);
}

#[test]
fn a_second_done_on_an_unchanged_tree_starts_no_process_for_a_verifier_that_passed() {
    let f = Fixture::new("reuse-second-done");
    fs::write(f.root.join("b.fail"), "").unwrap();
    let id = task(&f.repo, AGENT, "two verifiers", &["a", "b"]);

    let first = spawn_as(AGENT, ANK, &f.repo, &["done"]);
    assert_eq!(first.status.code(), Some(5), "b fails, so done refuses");
    assert_eq!((f.count("a"), f.count("b")), (1, 1));

    // Both runs are recorded, the pass and the failure, under the common
    // directory.
    let recorded = fs::read_dir(f.registry())
        .expect("the registry exists under <git-common-dir>/ank/runs")
        .count();
    assert_eq!(recorded, 2, "one entry per key run");

    fs::remove_file(f.root.join("b.fail")).unwrap();
    let second = spawn_as(AGENT, ANK, &f.repo, &["done"]);
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        (f.count("a"), f.count("b")),
        (1, 2),
        "a passed and is reused; b failed and a failure is never reused"
    );
    let err = String::from_utf8_lossy(&second.stderr);
    assert!(err.contains("a ... reused"), "{err}");

    let shown = run(ANK, &f.repo, &["show", &id]);
    assert_eq!(
        shown.matches("reused: local").count(),
        1,
        "only a carries reused:\n{shown}"
    );
    assert!(shown.contains(&format!("by: {AGENT}")), "{shown}");
    assert!(
        shown.contains("via: verifier"),
        "a reused entry keeps the route of the run that produced it:\n{shown}"
    );
}

#[test]
fn only_the_verifier_whose_inputs_changed_runs_again_and_fresh_runs_every_one() {
    let f = Fixture::new("reuse-edit-fresh");
    task(&f.repo, AGENT, "first", &["a", "b"]);
    run(ANK, &f.repo, &["done"]);
    commit(&f.repo, "first done");
    assert_eq!((f.count("a"), f.count("b")), (1, 1));

    task(&f.repo, AGENT, "second", &["a", "b"]);
    fs::write(f.repo.join("a/x"), "a, edited\n").unwrap();
    run(ANK, &f.repo, &["done"]);
    commit(&f.repo, "second done");
    assert_eq!(
        (f.count("a"), f.count("b")),
        (2, 1),
        "a's inputs changed, b's did not"
    );

    task(&f.repo, AGENT, "third", &["a", "b"]);
    run(ANK, &f.repo, &["done", "--fresh"]);
    assert_eq!(
        (f.count("a"), f.count("b")),
        (3, 2),
        "--fresh reuses nothing"
    );
}

#[test]
fn a_passing_run_older_than_reuse_max_age_is_never_reused() {
    let f = Fixture::new("reuse-max-age");
    task(&f.repo, AGENT, "first", &["a"]);
    run(ANK, &f.repo, &["done"]);
    commit(&f.repo, "first done");
    assert_eq!(f.count("a"), 1);

    // Forged for the second it names rather than waited for: every entry
    // recorded at the epoch is older than any reuse_max_age.
    for entry in fs::read_dir(f.registry()).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        let forged: String = text
            .lines()
            .map(|l| {
                if l.starts_with("at: ") {
                    "at: 0".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert_ne!(forged, text, "an entry carries its time as at:");
        fs::write(&path, forged + "\n").unwrap();
    }

    task(&f.repo, AGENT, "second", &["a"]);
    run(ANK, &f.repo, &["done"]);
    assert_eq!(f.count("a"), 2, "a run older than 24h is run again");
}

#[test]
fn the_registry_is_shared_by_two_worktrees_of_one_repository() {
    let f = Fixture::new("reuse-worktrees");
    task(&f.repo, AGENT, "first", &["a"]);
    run(ANK, &f.repo, &["done"]);
    commit(&f.repo, "first done");
    assert_eq!(f.count("a"), 1);

    let wt = f.root.join("wt");
    run(
        "git",
        &f.repo,
        &["worktree", "add", "-q", "-b", "other", wt.to_str().unwrap()],
    );
    let sibling = "test/verifier-reuse-sibling";
    let id = task(&wt, sibling, "in the sibling", &["a"]);
    run_as(sibling, ANK, &wt, &["done"]);
    assert_eq!(f.count("a"), 1, "the sibling reused the run of the first");

    let shown = run_as(sibling, ANK, &wt, &["show", &id]);
    assert!(shown.contains("reused: local"), "{shown}");
    assert!(
        shown.contains(&format!("by: {AGENT}")),
        "by names the identity that ran it, not the one reusing it:\n{shown}"
    );
}

#[test]
fn a_detached_attestation_is_never_reused() {
    let f = Fixture::new("reuse-attestation");
    let remote = f.root.join("remote.git");
    run(
        "git",
        &f.root,
        &["init", "-q", "--bare", remote.to_str().unwrap()],
    );
    run(
        "git",
        &f.repo,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    run("git", &f.repo, &["push", "-q", "origin", "main"]);

    let first = task(&f.repo, AGENT, "first", &["a"]);
    run(ANK, &f.repo, &["done"]);
    commit(&f.repo, "first done");
    run(
        ANK,
        &f.repo,
        &["attest", &first, "--proof", "test:ci://run-1", "--detached"],
    );
    assert_eq!(f.count("a"), 1);

    // The local run gone, the attestation is all that is left with the same
    // key, and it is not a source of reuse.
    fs::remove_dir_all(f.registry()).unwrap();
    task(&f.repo, AGENT, "second", &["a"]);
    run(ANK, &f.repo, &["done"]);
    assert_eq!(f.count("a"), 2, "nothing reused from refs/ank/proof");
}
