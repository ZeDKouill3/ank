//! `ank accept` takes several ids, and `--pr` carries them to a pull request
//! (ADR-2a63c0c87691, TASK-68db19c9ed93).
//!
//! Through the binary, over a bare origin reached by `file://`, with `gh`
//! replaced on `PATH` by a stub that records every invocation and answers
//! `auth status` as the fixture says.
//!
//! **The stub is a program, compiled once per run with the `rustc` that built
//! the suite**, and not a shell script: `Command::new("gh")` on Windows finds
//! `gh.exe` and never a script, so a script would have kept this suite off the
//! one platform where a path, a hook and a `PATH` lookup all behave
//! differently. A program runs the same everywhere.

use crate::scratch;

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";
const A: &str = "ADR-00000000000a";
const B: &str = "ADR-00000000000b";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("accept-pr-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [init]\n\tdefaultBranch = main\n[protocol \"file\"]\n\tallow = always\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn spawn(program: &str) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

const EXE: &str = std::env::consts::EXE_SUFFIX;

/// The stub's source: one line per invocation appended to the file
/// `ANK_TEST_GH_LOG` names, and `auth status` answered with the exit code
/// `ANK_TEST_GH_AUTH` holds. Both reach it through `ank`, which hands its
/// environment to the child it spawns.
const STUB: &str = r#"
use std::io::Write;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let log = std::env::var("ANK_TEST_GH_LOG").expect("ANK_TEST_GH_LOG");
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(log).unwrap();
    writeln!(f, "{}", args.join(" ")).unwrap();
    if args.len() >= 2 && args[0] == "auth" && args[1] == "status" {
        let code = std::env::var("ANK_TEST_GH_AUTH").ok().and_then(|v| v.parse().ok());
        std::process::exit(code.unwrap_or(0));
    }
}
"#;

/// The compiled stub, built once for the whole run.
fn stub() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let src = scratch::path("accept-pr-gh-stub.rs");
        let exe = scratch::path(&format!("accept-pr-gh-stub{EXE}"));
        fs::write(&src, STUB).unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let out = Command::new(rustc)
            .arg("--edition=2021")
            .arg("-o")
            .arg(&exe)
            .arg(&src)
            .output()
            .expect("rustc builds the suite, so it is there to build the stub");
        assert!(out.status.success(), "{}", text(&out.stderr));
        exe
    })
    .as_path()
}

/// A `PATH` reaching git and, when `bin` holds one, the stub, and never a `gh`
/// installed on the machine running the suite.
///
/// On Unix `bin` is the whole of it, with git linked in, because the
/// directory holding git commonly holds gh too (`/usr/bin`). On Windows a link
/// needs a privilege a runner may not have, and git lives in its own
/// directory, so the inherited `PATH` is kept minus every directory holding a
/// `gh.exe`.
fn search_path(bin: &Path) -> OsString {
    let inherited = std::env::var_os("PATH").expect("PATH is set");
    #[cfg(unix)]
    {
        let git = std::env::split_paths(&inherited)
            .map(|d| d.join("git"))
            .find(|p| p.is_file())
            .expect("git must be on PATH");
        std::os::unix::fs::symlink(git, bin.join("git")).unwrap();
        bin.as_os_str().to_owned()
    }
    #[cfg(not(unix))]
    {
        let kept =
            std::env::split_paths(&inherited).filter(|d| !d.join(format!("gh{EXE}")).is_file());
        std::env::join_paths(std::iter::once(bin.to_path_buf()).chain(kept)).unwrap()
    }
}

/// `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// A script git runs, executable where that is a permission.
fn hook(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn adr(id: &str) -> String {
    format!(
        "---\nid: {id}\ntype: adr\nslug: a-decision\ntitle: A decision\n\
         created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
         scope:\n  - src/**\nconstraint: |\n  A binding rule.\nschema: 3\nversion: 1\n---\n\nWhy.\n"
    )
}

/// How the stub answers `gh auth status`, and whether it is on `PATH` at all.
#[derive(Clone, Copy, PartialEq)]
enum Gh {
    Authenticated,
    Unauthenticated,
    Absent,
}

struct Corpus {
    repo: PathBuf,
    origin: PathBuf,
    path: OsString,
    gh: Gh,
    gh_log: PathBuf,
}

impl Corpus {
    /// Two proposed ADRs on `main`, pushed to a bare origin `main` tracks.
    fn new(what: &str, gh: Gh) -> Corpus {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        let origin = base.join("origin.git");
        let bin = base.join("bin");
        let gh_log = base.join("gh.log");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(&origin).unwrap();
        fs::create_dir_all(&bin).unwrap();
        git(&origin, &["init", "-q", "--bare", "-b", "main"]);
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        for id in [A, B] {
            fs::write(repo.join(".ank/entities").join(format!("{id}.md")), adr(id)).unwrap();
        }
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);
        git(&repo, &["remote", "add", "origin", &file_url(&origin)]);
        git(&repo, &["push", "-q", "-u", "origin", "main"]);

        if gh != Gh::Absent {
            fs::copy(stub(), bin.join(format!("gh{EXE}"))).unwrap();
        }
        let path = search_path(&bin);
        Corpus {
            repo,
            origin,
            path,
            gh,
            gh_log,
        }
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .env("ANK_AGENT", AGENT)
            .env("PATH", &self.path)
            .env("ANK_TEST_GH_LOG", &self.gh_log)
            .env(
                "ANK_TEST_GH_AUTH",
                if self.gh == Gh::Unauthenticated {
                    "1"
                } else {
                    "0"
                },
            )
            .current_dir(&self.repo)
            .output()
            .expect("the binary must have been built")
    }

    fn gh_calls(&self) -> Vec<String> {
        fs::read_to_string(&self.gh_log)
            .unwrap_or_default()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn subject(&self, rev: &str) -> String {
        git(&self.repo, &["log", "-1", "--format=%s", rev])
    }

    fn origin_ref(&self, name: &str) -> Option<String> {
        let out = spawn("git")
            .args(["rev-parse", "--verify", "-q", name])
            .current_dir(&self.origin)
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| text(&out.stdout).trim().to_string())
    }

    /// Everything a refusal must leave as it found it.
    fn state(&self) -> (String, String, Vec<String>) {
        let head = git(&self.repo, &["rev-parse", "HEAD"]);
        let status = git(&self.repo, &["status", "--porcelain"]);
        let branches = git(&self.repo, &["branch", "--format=%(refname)"])
            .lines()
            .map(str::to_string)
            .collect();
        (head, status, branches)
    }
}

#[test]
fn several_ids_are_several_ratifications_and_nothing_is_pushed() {
    let c = Corpus::new("accept-pr-plain", Gh::Authenticated);
    let pushed = c.origin_ref("refs/heads/main");

    let out = c.ank(&["accept", A, B]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    assert_eq!(c.subject("HEAD~1"), format!("ratify {A}"));
    assert_eq!(c.subject("HEAD"), format!("ratify {B}"));
    assert_eq!(
        c.origin_ref("refs/heads/main"),
        pushed,
        "a plain accept pushes nothing"
    );
    assert_eq!(c.origin_ref(&format!("refs/heads/ratify/{A}")), None);
    assert!(
        c.gh_calls().is_empty(),
        "gh is not called: {:?}",
        c.gh_calls()
    );
}

#[test]
fn pr_carries_the_ratifications_to_a_pull_request_and_never_merges() {
    let c = Corpus::new("accept-pr-carries", Gh::Authenticated);
    let upstream = git(&c.repo, &["rev-parse", "refs/remotes/origin/main"]);

    let out = c.ank(&["accept", A, B, "--pr"]);
    let stdout = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{}{stdout}", text(&out.stderr));

    let branch = format!("ratify/{A}");
    let tip = git(&c.repo, &["rev-parse", &format!("refs/heads/{branch}")]);
    assert_eq!(c.subject(&tip), format!("ratify {B}"));
    assert_eq!(c.subject(&format!("{tip}~1")), format!("ratify {A}"));
    assert_eq!(
        c.origin_ref(&format!("refs/heads/{branch}")),
        Some(tip.clone()),
        "the branch is on origin, at the second ratification"
    );

    assert_eq!(
        git(&c.repo, &["rev-parse", "refs/heads/main"]),
        git(&c.repo, &["rev-parse", "refs/remotes/origin/main"]),
        "the local default branch equals its upstream"
    );
    assert_eq!(git(&c.repo, &["rev-parse", "HEAD"]), upstream);
    assert_eq!(
        git(&c.repo, &["status", "--porcelain"]),
        "",
        "the tree is clean on the reset default branch"
    );

    let calls = c.gh_calls();
    let creates: Vec<&String> = calls
        .iter()
        .filter(|l| l.starts_with("pr create"))
        .collect();
    assert_eq!(creates.len(), 1, "exactly one pr create: {calls:?}");
    assert!(creates[0].contains("--base main"), "{calls:?}");
    assert!(
        creates[0].contains(&format!("--head {branch}")),
        "{calls:?}"
    );
    assert!(
        !calls.iter().any(|l| l.starts_with("pr merge")),
        "never merges: {calls:?}"
    );
    assert!(
        stdout.contains(&format!("gh pr merge {branch} --merge")),
        "{stdout}"
    );
}

/// The three prerequisites, each refused before the first commit, with the
/// code ADR-2a63c0c87691 gives it and the tree as it was.
#[test]
fn pr_refuses_before_any_commit_when_a_prerequisite_is_missing() {
    let cases: [(&str, Gh, fn(&Corpus), i32, &str); 4] = [
        (
            "accept-pr-no-upstream",
            Gh::Authenticated,
            |c| {
                git(&c.repo, &["branch", "--unset-upstream", "main"]);
            },
            9,
            "no upstream",
        ),
        (
            "accept-pr-gh-absent",
            Gh::Absent,
            |_| {},
            9,
            "gh is not on PATH",
        ),
        (
            "accept-pr-gh-unauthenticated",
            Gh::Unauthenticated,
            |_| {},
            9,
            "gh auth login",
        ),
        (
            "accept-pr-dirty",
            Gh::Authenticated,
            |c| {
                fs::write(c.repo.join("src/lib.rs"), "pub fn g() {}\n").unwrap();
            },
            7,
            "src/lib.rs",
        ),
    ];
    for (what, gh, arrange, code, why) in cases {
        let c = Corpus::new(what, gh);
        arrange(&c);
        let before = c.state();

        let out = c.ank(&["accept", A, B, "--pr"]);
        assert_eq!(
            out.status.code(),
            Some(code),
            "{what}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        assert!(
            text(&out.stderr).contains(why),
            "{what}: the refusal says why: {}",
            text(&out.stderr)
        );
        assert_eq!(c.state(), before, "{what}: the tree is unchanged");
        assert!(
            !c.gh_calls().iter().any(|l| l.starts_with("pr ")),
            "{what}: {:?}",
            c.gh_calls()
        );
    }
}

#[test]
fn a_refused_push_leaves_the_commits_on_the_branch_and_names_the_rest() {
    let c = Corpus::new("accept-pr-push-refused", Gh::Authenticated);
    let hook = c.origin.join("hooks/pre-receive");
    self::hook(&hook, "#!/bin/sh\necho refused by policy >&2\nexit 1\n");

    let out = c.ank(&["accept", A, B, "--pr"]);
    let said = format!("{}{}", text(&out.stdout), text(&out.stderr));
    assert_ne!(out.status.code(), Some(0), "{said}");

    let branch = format!("ratify/{A}");
    let tip = git(&c.repo, &["rev-parse", &format!("refs/heads/{branch}")]);
    assert_eq!(c.subject(&tip), format!("ratify {B}"));
    assert_eq!(c.subject(&format!("{tip}~1")), format!("ratify {A}"));
    assert_eq!(c.origin_ref(&format!("refs/heads/{branch}")), None);
    assert!(
        said.contains(&format!("git push -u origin {branch}")),
        "names the command that finishes the job: {said}"
    );
    assert!(
        !c.gh_calls().iter().any(|l| l.starts_with("pr ")),
        "{:?}",
        c.gh_calls()
    );
}

#[test]
fn the_ratifying_page_leads_with_accept_pr_and_keeps_the_manual_recipe() {
    let page =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/ratifying.md"))
            .unwrap();
    let pr = page.find("--pr").expect("the page names accept --pr");
    let manual = page
        .find("git branch ratify/")
        .expect("the page keeps the manual recipe");
    assert!(pr < manual, "accept --pr comes first");
    assert!(page.contains("git reset --hard origin/main"));
}

/// Several ids answer one document, a row per ratification, and `branch` and
/// `merge` are null when nothing was carried.
#[test]
fn several_ids_answer_one_document_with_a_row_each() {
    let c = Corpus::new("accept-pr-json", Gh::Authenticated);

    let out = c.ank(&["accept", A, B, "--json"]);
    let doc = text(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    assert_eq!(doc.lines().count(), 1, "one document: {doc}");
    let rows = doc.split("\"accepted\":[").nth(1).expect(&doc);
    assert!(rows.contains(&format!("\"id\":\"{A}\"")), "{doc}");
    assert!(rows.contains(&format!("\"id\":\"{B}\"")), "{doc}");
    assert!(doc.contains("\"branch\":null"), "{doc}");
    assert!(doc.contains("\"merge\":null"), "{doc}");
}
