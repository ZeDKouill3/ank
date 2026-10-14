//! A repository whose origin does not fetch `refs/ank/*`: `init` says why it
//! adds nothing while there is no origin, and once there is one, the first verb
//! that asks adds the refspec and fetches `refs/ank/*` (TASK-623d80886c2f,
//! issue #499, ADR-1b030fe70442, TASK-b55c33df5aa3).
//!
//! Measured with ank 0.8.0 in a scratch repository: `git init -b main && ank
//! init` exited 0 and said nothing about the refspec, `git remote add origin
//! <url>` left `remote.origin.fetch` holding `+refs/heads/*:refs/remotes/origin/*`
//! alone, and neither `status` nor `check` printed a line about it. Claims and
//! completion refs were then never fetched, and nothing said so. A fresh clone
//! is the same state by construction: the refspec lives in `.git/config`, which
//! a clone never carries. Reporting the gap was the first answer; measured on
//! this repository at fb17a26a, a fresh clone still read 214 attested tasks as
//! unproven until somebody acted on the report, so the report became a repair.
//!
//! `init` still writes nothing while origin has no URL, and neither does the
//! repair -- that is what keeps `git remote add origin` working
//! (TASK-f067ae7c84ff, tests/init_origin.rs).
//!
//! **Refspecs, not exit codes**: each step asserts on `git config --get-all
//! remote.origin.fetch`.

use crate::scratch;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const ANK_REFSPEC: &str = "+refs/ank/*:refs/ank/*";
const HEADS_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";

/// A global and system git config of this suite's own, so a developer's
/// `~/.gitconfig` adds nothing to the remote under test.
fn isolated_git_config() -> &'static Path {
    static CONFIG: OnceLock<PathBuf> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let p = scratch::path("init-refspec-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.invalid\n\
             [init]\n\tdefaultBranch = main\n",
        )
        .unwrap();
        p
    })
}

/// Git-for-Windows' path conversion switches are dropped for the reason
/// `tests/status.rs` gives.
fn spawn(program: impl AsRef<OsStr>, dir: &Path) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", "init@fixture")
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("MSYS2_ARG_CONV_EXCL");
    c
}

fn run(mut c: Command) -> (i32, String) {
    let out = c.output().expect("the process must start");
    let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
    said.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().expect("must not be signalled"), said)
}

fn git(dir: &Path, args: &[&str]) -> String {
    let mut c = spawn("git", dir);
    c.args(args);
    let (code, said) = run(c);
    assert_eq!(code, 0, "git {args:?}: {said}");
    said.trim().to_string()
}

fn ank(dir: &Path, args: &[&str]) -> (i32, String) {
    let mut c = spawn(ANK, dir);
    c.args(args);
    run(c)
}

fn fetch_refspecs(dir: &Path) -> Vec<String> {
    let mut c = spawn("git", dir);
    c.args(["config", "--get-all", "remote.origin.fetch"]);
    let (_, said) = run(c);
    said.lines().map(str::to_string).collect()
}

/// The line of `said` that reports the missing refspec, if any.
fn refspec_line(said: &str) -> Option<&str> {
    said.lines()
        .find(|l| l.contains(ANK_REFSPEC) && l.contains("remote.origin.fetch"))
}

/// The command a line names in its closing parenthesis, split into words.
fn named_command(line: &str) -> Vec<String> {
    let inner = line
        .trim_end()
        .strip_suffix(')')
        .and_then(|l| l.rsplit_once('('))
        .map(|(_, c)| c)
        .unwrap_or_else(|| panic!("no command named in: {line}"));
    inner.split_whitespace().map(str::to_string).collect()
}

/// One line of `err` names the refspec, and it says the refspec was added.
fn one_repair_line(err: &str) {
    let said: Vec<&str> = err.lines().filter(|l| l.contains(ANK_REFSPEC)).collect();
    assert_eq!(said.len(), 1, "one stderr line names the repair: {err}");
    assert!(said[0].contains("added"), "{err}");
}

fn neither_reports_the_gap(dir: &Path) {
    let (code, said) = ank(dir, &["status"]);
    assert_eq!(code, 0, "{said}");
    assert!(refspec_line(&said).is_none(), "{said}");
    let (code, said) = ank(dir, &["check"]);
    assert_eq!(code, 0, "{said}");
    assert!(refspec_line(&said).is_none(), "{said}");
}

/// `init` says why it added nothing while there is no origin; once origin
/// exists, the first verb that asks repairs the refspec instead of reporting
/// it (ADR-1b030fe70442), in the repository `init` ran in and in a fresh clone.
#[test]
fn a_refspec_init_could_not_add_is_added_by_the_first_verb_once_origin_exists() {
    let base = scratch::dir("init-refspec-sequence");
    let repo = base.join("repo");
    let bare = base.join("origin.git");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bare).unwrap();
    git(&repo, &["init", "-q", "-b", "main", "."]);
    git(&bare, &["init", "-q", "--bare", "."]);

    // init says it added nothing, why, and what adds it once origin exists.
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    let line = said
        .lines()
        .find(|l| l.contains(ANK_REFSPEC))
        .unwrap_or_else(|| panic!("init is silent on the refspec: {said}"));
    assert!(line.contains("not added"), "{line}");
    assert!(line.contains("no remote named origin"), "{line}");
    assert_eq!(named_command(line), vec!["ank", "init"]);
    assert!(fetch_refspecs(&repo).is_empty());
    // Nothing to repair yet: there is no origin to fetch from.
    neither_reports_the_gap(&repo);

    // The remote is added, and git still accepts it.
    let mut c = spawn("git", &repo);
    c.args(["remote", "add", "origin", &bare.to_string_lossy()]);
    let (code, out) = run(c);
    assert_eq!(code, 0, "git remote add origin failed: {out}");
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC]);

    let (code, out, err) = ank_traced(&repo, &["status"], None);
    assert_eq!(code, 0, "{out}{err}");
    one_repair_line(&err);
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    neither_reports_the_gap(&repo);

    // A fresh clone of a repository whose origin carries refs/ank/*: the
    // refspec lives in .git/config, so the clone has only git's own, and here
    // `check` is the first verb to ask.
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "ank init"]);
    git(&repo, &["update-ref", "refs/ank/fixture/ref", "HEAD"]);
    git(
        &repo,
        &["push", "-q", "origin", "main", "refs/ank/fixture/ref"],
    );
    let clone = base.join("clone");
    git(
        &base,
        &[
            "clone",
            "-q",
            &bare.to_string_lossy(),
            &clone.to_string_lossy(),
        ],
    );
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC]);
    let (code, out, err) = ank_traced(&clone, &["check"], None);
    assert_eq!(code, 0, "{out}{err}");
    one_repair_line(&err);
    assert!(refspec_line(&out).is_none(), "{out}");
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    // And what the refspec is for, brought by the repair itself.
    let refs = git(&clone, &["for-each-ref", "--format=%(refname)", "refs/ank"]);
    assert!(refs.lines().any(|r| r == "refs/ank/fixture/ref"), "{refs}");
    neither_reports_the_gap(&clone);
}

#[test]
fn help_init_no_longer_says_the_refspec_is_always_added() {
    let dir = scratch::dir("init-refspec-help");
    let (code, said) = ank(&dir, &["help", "init"]);
    assert_eq!(code, 0, "{said}");
    assert!(!said.contains("adds the refs/ank/* refspec;"), "{said}");
    assert!(said.contains("once origin"), "{said}");
}

// ---------------------------------------------------------------------------
// The repair (ADR-1b030fe70442, TASK-b55c33df5aa3)
// ---------------------------------------------------------------------------

/// The same invocation as [`ank`], with stdout and stderr kept apart and
/// `GIT_TRACE2_EVENT` pointed at `trace` when one is given.
fn ank_traced(dir: &Path, args: &[&str], trace: Option<&Path>) -> (i32, String, String) {
    let mut c = spawn(ANK, dir);
    c.args(args);
    if let Some(t) = trace {
        c.env("GIT_TRACE2_EVENT", t);
    }
    let out = c.output().expect("the process must start");
    (
        out.status.code().expect("must not be signalled"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The git processes a trace records, each as its argv joined by spaces. One
/// `start` event per process, written by git itself: no shim on `PATH`, for
/// the reason `checks_git_cost_does_not_grow_with_the_number_of_dead_scopes`
/// gives in tests/cli.rs.
fn git_starts(trace: &Path) -> Vec<String> {
    let text = fs::read_to_string(trace).unwrap_or_default();
    text.lines()
        .filter(|l| l.contains("\"event\":\"start\""))
        .map(|l| {
            let argv = l.split("\"argv\":[").nth(1).unwrap_or("");
            let argv = argv.split(']').next().unwrap_or("");
            argv.replace('"', "").replace(',', " ")
        })
        .collect()
}

fn ank_refs(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = git(dir, &["for-each-ref", "--format=%(refname)", "refs/ank"])
        .lines()
        .map(str::to_string)
        .collect();
    v.sort();
    v
}

fn ank_refs_on(dir: &Path, remote: &Path) -> Vec<String> {
    let mut v: Vec<String> = git(dir, &["ls-remote", &remote.to_string_lossy(), "refs/ank/*"])
        .lines()
        .filter_map(|l| l.split_once('\t').map(|(_, r)| r.to_string()))
        .collect();
    v.sort();
    v
}

/// A bare origin carrying a done task and its `refs/ank/proof/<id>`, and a
/// fresh clone of it whose `remote.origin.fetch` holds the branch refspec
/// alone: every clone's state, because a refspec lives in `.git/config`.
/// Returns the base directory, the origin, the clone and the task id.
fn a_fresh_clone_of_an_attested_origin(what: &str) -> (PathBuf, PathBuf, PathBuf, String) {
    let base = scratch::dir(what);
    let repo = base.join("repo");
    let bare = base.join("origin.git");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bare).unwrap();
    git(&repo, &["init", "-q", "-b", "main", "."]);
    git(&bare, &["init", "-q", "--bare", "-b", "main", "."]);
    git(&repo, &["remote", "add", "origin", &bare.to_string_lossy()]);
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::write(
        repo.join("src/lib.rs"),
        "// the code the task scopes
",
    )
    .unwrap();
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    let (code, said) = ank(&repo, &["config", "default_branch", "main"]);
    assert_eq!(code, 0, "{said}");
    let (code, said) = ank(
        &repo,
        &[
            "new",
            "task",
            "--title",
            "A finished task",
            "--criteria",
            "A verifiable criterion.",
            "--scope",
            "src/**",
            "--no-verify",
        ],
    );
    assert_eq!(code, 0, "{said}");
    let id = said
        .split_whitespace()
        .find(|w| w.starts_with("TASK-"))
        .unwrap_or_else(|| panic!("no id in: {said}"))
        .to_string();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the task"]);
    let sha = git(&repo, &["rev-parse", "HEAD"]);
    let (code, said) = ank(&repo, &["claim", &id]);
    assert_eq!(code, 0, "{said}");
    let (code, said) = ank(&repo, &["done", &id, "--proof", &format!("commit:{sha}")]);
    assert_eq!(code, 0, "{said}");
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "done"]);
    git(&repo, &["push", "-q", "origin", "main"]);
    let (code, said) = ank(
        &repo,
        &["attest", &id, "--proof", "test:4242", "--detached"],
    );
    assert_eq!(code, 0, "{said}");
    assert!(
        ank_refs_on(&repo, &bare)
            .iter()
            .any(|r| r == &format!("refs/ank/proof/{id}")),
        "the fixture's origin must carry the proof ref"
    );

    let clone = base.join("clone");
    git(
        &base,
        &[
            "clone",
            "-q",
            &bare.to_string_lossy(),
            &clone.to_string_lossy(),
        ],
    );
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC]);
    assert!(ank_refs(&clone).is_empty(), "{:?}", ank_refs(&clone));
    (base, bare, clone, id)
}

#[test]
fn the_first_status_in_a_fresh_clone_adds_the_refspec_and_fetches_refs_ank_once() {
    let (base, bare, clone, id) = a_fresh_clone_of_an_attested_origin("refspec-repair");

    let (code, _, err) = ank_traced(&clone, &["status"], None);
    assert_eq!(code, 0, "{err}");
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    let said: Vec<&str> = err.lines().filter(|l| l.contains(ANK_REFSPEC)).collect();
    assert_eq!(said.len(), 1, "one stderr line names the repair: {err}");
    assert!(
        said[0].contains("added") && said[0].contains("fetched"),
        "the line names the refspec added and the fetch: {err}"
    );
    assert_eq!(ank_refs(&clone), ank_refs_on(&clone, &bare));

    let (code, out, err) = ank_traced(&clone, &["check"], None);
    assert_eq!(code, 0, "{out}{err}");
    assert!(
        !out.lines()
            .any(|l| l.contains(&id) && l.contains("done with no test proof")),
        "the attestation origin holds is read: {out}"
    );
    assert!(refspec_line(&out).is_none(), "{out}");
    assert!(refspec_line(&err).is_none(), "{err}");

    let trace = base.join("second-status.trace");
    let (code, _, err) = ank_traced(&clone, &["status"], Some(&trace));
    assert_eq!(code, 0, "{err}");
    let starts = git_starts(&trace);
    assert!(
        !starts.is_empty(),
        "the trace recorded nothing, so this measures nothing"
    );
    let fetches: Vec<&String> = starts.iter().filter(|s| s.contains(" fetch")).collect();
    assert!(fetches.is_empty(), "a second status fetched: {fetches:?}");
    assert!(!err.contains(ANK_REFSPEC), "{err}");
}

#[test]
fn an_unreachable_origin_still_gets_the_refspec_and_the_next_fetch_brings_the_refs() {
    let (base, bare, clone, _) = a_fresh_clone_of_an_attested_origin("refspec-offline");
    let gone = base.join("nowhere.git");
    git(
        &clone,
        &["config", "remote.origin.url", &gone.to_string_lossy()],
    );

    let (code, out, err) = ank_traced(&clone, &["status"], None);
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    // The refspec is now present, so a second run is the verb as it is with
    // the refspec there, against the same unreachable origin.
    let (again, out2, err2) = ank_traced(&clone, &["status"], None);
    assert_eq!(code, again, "{out}{err}\n---\n{out2}{err2}");
    assert_eq!(code, 0, "{out}{err}");

    git(
        &clone,
        &["config", "remote.origin.url", &bare.to_string_lossy()],
    );
    git(&clone, &["fetch", "-q", "origin"]);
    assert_eq!(ank_refs(&clone), ank_refs_on(&clone, &bare));
}

#[test]
fn with_no_origin_nothing_is_written_and_git_remote_add_still_works() {
    let base = scratch::dir("refspec-no-origin");
    let repo = base.join("repo");
    let bare = base.join("origin.git");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bare).unwrap();
    git(&repo, &["init", "-q", "-b", "main", "."]);
    git(&bare, &["init", "-q", "--bare", "."]);
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    for verb in ["status", "check"] {
        let (code, said) = ank(&repo, &[verb]);
        assert_eq!(code, 0, "{verb}: {said}");
        assert!(fetch_refspecs(&repo).is_empty(), "{verb} wrote a refspec");
    }
    let mut c = spawn("git", &repo);
    c.args(["remote", "add", "origin", &bare.to_string_lossy()]);
    let (code, out) = run(c);
    assert_eq!(code, 0, "git remote add origin failed: {out}");
}

#[test]
fn help_and_version_start_no_git_process() {
    let base = scratch::dir("refspec-no-git");
    for args in [&["help"][..], &["--version"][..]] {
        let trace = base.join(format!("{}.trace", args[0].trim_start_matches('-')));
        let (code, out, err) = ank_traced(&base, args, Some(&trace));
        assert_eq!(code, 0, "{args:?}: {out}{err}");
        assert_eq!(git_starts(&trace), Vec::<String>::new(), "{args:?}");
    }
}
