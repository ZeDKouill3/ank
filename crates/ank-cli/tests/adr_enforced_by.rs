//! An ADR names what enforces it, and context serves such a rule by its title
//! (TASK-e3bb16dd4ba1, ADR-25584c56c1a7).
//!
//! Every case through the binary: `new adr --enforced-by` writes the field and
//! `show` prints it; `amend --enforced-by` and `--drop-enforced-by` write it on
//! a proposed ADR and on an accepted one, whose ratification anchor still
//! holds; `check` signals an entry whose path no tracked file matches and one
//! whose symbol the file does not contain; `context` serves an enforced rule as
//! one line in both modes, its constraint nowhere; and the over-constrained
//! charge of a task stops counting it.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

/// What holds the rule in every case below: a tracked file, and a function it
/// carries.
const HELD: &str = "src/tests.rs#logs_every_module";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("enforced-by-gitconfig");
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

fn spawn(program: &str) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env_remove("NO_COLOR")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

struct Repo(PathBuf);

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            dir.join("src/tests.rs"),
            "#[test]\nfn logs_every_module() {}\n",
        )
        .unwrap();
        let r = Repo(dir);
        r.git(&["init", "-q", "-b", "main"]);
        fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r.commit("seed");
        r
    }

    fn git(&self, args: &[&str]) {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.0)
            .current_dir(scratch::root())
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank(args);
        assert_eq!(
            code(&out),
            0,
            "ank {args:?}: {}{}",
            stdout(&out),
            stderr(&out)
        );
        stdout(&out)
    }

    fn created(&self, args: &[&str]) -> String {
        self.ok(args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    fn adr(&self, title: &str, constraint: &str, extra: &[&str]) -> String {
        let mut args = vec![
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            constraint,
        ];
        args.extend_from_slice(extra);
        self.created(&args)
    }

    fn task(&self, title: &str) -> String {
        self.created(&[
            "new",
            "task",
            "--title",
            title,
            "--scope",
            "src/**",
            "--criteria",
            "It holds.",
            "--no-verify",
        ])
    }

    /// Committed, then ratified: `accept` works on the default branch and on
    /// what is committed there.
    fn accept(&self, id: &str) {
        self.commit(&format!("propose {id}"));
        self.ok(&["accept", id]);
    }

    fn show(&self, id: &str) -> String {
        self.ok(&["show", id])
    }

    fn check(&self) -> String {
        let out = self.ank(&["check"]);
        assert!(
            [0, 8].contains(&code(&out)),
            "check: {}{}",
            stdout(&out),
            stderr(&out)
        );
        stdout(&out)
    }
}

/// The `enforced_by:` block `show` prints of an ADR, its entries in order, or
/// `None` when the ADR declares nothing.
fn enforced_by(text: &str) -> Option<Vec<String>> {
    let mut lines = text.lines().skip_while(|l| *l != "enforced_by:");
    lines.next()?;
    Some(
        lines
            .map_while(|l| l.strip_prefix("  - "))
            .map(str::to_string)
            .collect(),
    )
}

/// The lines `check` prints about one entity, in order.
fn about(out: &str, id: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.contains(id))
        .map(str::to_string)
        .collect()
}

fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines of a page naming `id`.
fn lines_naming<'a>(page: &'a str, id: &str) -> Vec<&'a str> {
    page.lines().filter(|l| l.contains(short(id))).collect()
}

#[test]
fn new_adr_writes_enforced_by_and_show_prints_it() {
    let r = Repo::new("enforced-by-new");
    let other = "src/main.rs#main";
    let a = r.adr(
        "Every module logs",
        "Every module logs.",
        &["--enforced-by", HELD, "--enforced-by", other],
    );
    assert_eq!(
        enforced_by(&r.show(&a)),
        Some(vec![HELD.to_string(), other.to_string()])
    );

    // Without the flag, the field is omitted rather than written empty.
    let b = r.adr("Nothing holds this", "A rule.", &[]);
    assert_eq!(enforced_by(&r.show(&b)), None);

    // An entry that is not <path>#<symbol> is refused when it is typed.
    for bad in ["src/tests.rs", "#logs_every_module", "src/tests.rs#"] {
        let out = r.ank(&[
            "new",
            "adr",
            "--title",
            "t",
            "--scope",
            "src/**",
            "--constraint",
            "c",
            "--enforced-by",
            bad,
        ]);
        assert_eq!(code(&out), 1, "{bad}: {}", stderr(&out));
        assert!(stderr(&out).contains("<path>#<symbol>"), "{}", stderr(&out));
    }

    // Only an ADR binds, so only an ADR is enforced.
    for kind in ["task", "spec"] {
        let out = r.ank(&[
            "new",
            kind,
            "--title",
            "t",
            "--scope",
            "src/**",
            "--enforced-by",
            HELD,
        ]);
        assert_eq!(code(&out), 1, "{kind}: {}", stderr(&out));
        assert!(
            stderr(&out).contains("--enforced-by applies to an ADR"),
            "{kind}: {}",
            stderr(&out)
        );
    }
}

#[test]
fn the_path_of_an_entry_is_normalised_as_a_scope_is() {
    let r = Repo::new("enforced-by-normalised");
    // The forms a shell completes, each stored as the one path `check` and
    // every other platform read back.
    let a = r.adr(
        "Every module logs",
        "Every module logs.",
        &["--enforced-by", "./src/tests.rs#logs_every_module"],
    );
    assert_eq!(enforced_by(&r.show(&a)), Some(vec![HELD.to_string()]));
    r.ok(&["amend", &a, "--enforced-by", "src\\main.rs#main"]);
    assert_eq!(
        enforced_by(&r.show(&a)),
        Some(vec![HELD.to_string(), "src/main.rs#main".to_string()])
    );
    // And dropped by the same form it may have been typed in.
    r.ok(&["amend", &a, "--drop-enforced-by", "./src/main.rs#main"]);
    assert_eq!(enforced_by(&r.show(&a)), Some(vec![HELD.to_string()]));
}

#[test]
fn amend_writes_enforced_by_on_a_proposed_adr() {
    let r = Repo::new("enforced-by-amend-proposed");
    let a = r.adr("Every module logs", "Every module logs.", &[]);

    let out = r.ok(&["amend", &a, "--enforced-by", HELD]);
    assert!(out.contains(&format!("+enforced_by {HELD}")), "{out}");
    assert_eq!(enforced_by(&r.show(&a)), Some(vec![HELD.to_string()]));

    let out = r.ok(&["amend", &a, "--drop-enforced-by", HELD]);
    assert!(out.contains(&format!("-enforced_by {HELD}")), "{out}");
    assert_eq!(enforced_by(&r.show(&a)), None);

    // Dropping what is not there is refused, not ignored.
    let out = r.ank(&["amend", &a, "--drop-enforced-by", HELD]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("is not enforced by"),
        "{}",
        stderr(&out)
    );

    // A task does not carry the field.
    let t = r.task("Some work");
    let out = r.ank(&["amend", &t, "--enforced-by", HELD]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("enforced_by applies to an ADR"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn amend_writes_enforced_by_on_an_accepted_adr_and_its_anchor_still_holds() {
    let r = Repo::new("enforced-by-amend-accepted");
    let a = r.adr("Every module logs", "Every module logs.", &[]);
    r.accept(&a);
    let before = about(&r.check(), &a);
    assert!(
        !before
            .iter()
            .any(|l| l.contains("altered since ratification")),
        "{before:?}"
    );

    r.ok(&["amend", &a, "--enforced-by", HELD]);
    assert_eq!(enforced_by(&r.show(&a)), Some(vec![HELD.to_string()]));
    assert!(r.show(&a).contains("\nstatus: accepted\n"));
    r.commit("enforce");
    assert_eq!(about(&r.check(), &a), before);

    r.ok(&["amend", &a, "--drop-enforced-by", HELD]);
    assert_eq!(enforced_by(&r.show(&a)), None);
    r.commit("drop");
    assert_eq!(about(&r.check(), &a), before);
}

#[test]
fn check_signals_a_dead_path_and_an_absent_symbol_naming_the_entry() {
    let r = Repo::new("enforced-by-check");
    let dead_path = "src/gone.rs#logs_every_module";
    let absent_symbol = "src/tests.rs#logs_nothing";
    // A symbol that only prefixes another is not the symbol.
    let partial = "src/tests.rs#logs_every";
    let a = r.adr(
        "Every module logs",
        "Every module logs.",
        &[
            "--enforced-by",
            HELD,
            "--enforced-by",
            dead_path,
            "--enforced-by",
            absent_symbol,
            "--enforced-by",
            partial,
        ],
    );
    r.commit("adr");
    let out = r.check();
    let lines = about(&out, &a);

    let naming = |entry: &str| -> Vec<&String> {
        lines
            .iter()
            .filter(|l| l.contains(entry) && l.contains("enforced_by"))
            .collect()
    };
    let dead = naming(dead_path);
    assert_eq!(dead.len(), 1, "{out}");
    assert!(dead[0].starts_with("signal"), "{out}");
    assert!(dead[0].contains("no tracked file"), "{out}");

    let absent = naming(absent_symbol);
    assert_eq!(absent.len(), 1, "{out}");
    assert!(absent[0].starts_with("signal"), "{out}");
    assert!(absent[0].contains("does not contain"), "{out}");

    assert_eq!(naming(partial).len(), 1, "{out}");
    // The entry that holds is reported nowhere.
    assert!(
        !lines
            .iter()
            .any(|l| l.contains(&format!("'{HELD}'")) || l.ends_with(HELD)),
        "{out}"
    );

    // A file on disk that git ignores is not a tracked file.
    fs::write(r.0.join(".gitignore"), ".ank/index.db*\nsrc/local.rs\n").unwrap();
    fs::write(r.0.join("src/local.rs"), "fn here() {}\n").unwrap();
    r.ok(&["amend", &a, "--enforced-by", "src/local.rs#here"]);
    r.commit("ignored");
    let out = r.check();
    assert!(
        about(&out, &a)
            .iter()
            .any(|l| l.contains("src/local.rs#here") && l.contains("no tracked file")),
        "{out}"
    );
}

/// The corpus the context cases start from: one accepted ADR that a test holds,
/// one nothing holds, both binding `src/**`, and a task under both.
struct Pair {
    repo: Repo,
    held: String,
    loose: String,
    task: String,
}

const HELD_RULE: &str = "Every module under src logs what it opens, and the log names the file.";
const LOOSE_RULE: &str =
    "Every module under src tests what it parses, and the test names the case.";

fn pair(what: &str) -> Pair {
    let repo = Repo::new(what);
    let held = repo.adr("Every module logs", HELD_RULE, &["--enforced-by", HELD]);
    repo.accept(&held);
    let loose = repo.adr("Every module tests", LOOSE_RULE, &[]);
    repo.accept(&loose);
    let task = repo.task("Some work under src");
    repo.commit("task");
    Pair {
        repo,
        held,
        loose,
        task,
    }
}

#[test]
fn context_serves_an_enforced_rule_as_one_line_in_orientation_mode() {
    let p = pair("enforced-by-context-orientation");
    let page = p.repo.ok(&["context", "src/main.rs"]);
    let held = lines_naming(&page, &p.held);
    assert_eq!(held.len(), 1, "{page}");
    assert!(held[0].contains("Every module logs"), "{page}");
    assert!(
        held[0].contains(HELD),
        "the line names what holds it: {page}"
    );
    assert!(!page.contains(HELD_RULE), "{page}");
    let loose = lines_naming(&page, &p.loose);
    assert_eq!(loose.len(), 1, "{page}");
    assert!(!loose[0].contains("enforced by"), "{page}");

    let doc = p.repo.ok(&["context", "src/main.rs", "--json"]);
    assert!(!doc.contains(HELD_RULE), "{doc}");
    assert!(
        doc.contains(&format!(
            "\"constraint\":\"\",\"home\":null,\"amended_by\":[],\"enforced_by\":[\"{HELD}\"]"
        )),
        "{doc}"
    );
    // A rule nothing holds is untouched, and says so with an empty list.
    assert!(
        doc.contains(&format!(
            "\"constraint\":\"{LOOSE_RULE}\",\"home\":null,\"amended_by\":[],\"enforced_by\":[]"
        )),
        "{doc}"
    );
}

#[test]
fn context_serves_an_enforced_rule_as_one_line_in_execution_mode() {
    let p = pair("enforced-by-context-execution");
    p.repo.ok(&["claim", &p.task]);
    let page = p.repo.ok(&["context"]);
    let held = lines_naming(&page, &p.held);
    assert_eq!(held.len(), 1, "{page}");
    assert!(held[0].contains("Every module logs"), "{page}");
    assert!(held[0].contains(HELD), "{page}");
    assert!(!page.contains(HELD_RULE), "{page}");
    // The rule nothing holds is still served whole.
    assert!(page.contains(LOOSE_RULE), "{page}");

    let doc = p.repo.ok(&["context", "--json"]);
    assert!(!doc.contains(HELD_RULE), "{doc}");
    assert!(doc.contains(LOOSE_RULE), "{doc}");
    assert!(
        doc.contains(&format!("\"enforced_by\":[\"{HELD}\"]")),
        "{doc}"
    );
}

/// The `charges N characters` line `check` prints for `adr` under the
/// over-constrained finding of `task`, as N, or `None` when it is not charged.
fn charge(out: &str, adr: &str) -> Option<usize> {
    out.lines()
        .find(|l| l.contains(adr) && l.contains(" charges "))
        .map(|l| {
            l.split(" charges ")
                .nth(1)
                .and_then(|r| r.split_whitespace().next())
                .and_then(|n| n.parse().ok())
                .expect("charges <n> characters")
        })
}

/// The total of the over-constrained finding on `task`.
fn weight(out: &str, task: &str) -> Option<usize> {
    out.lines()
        .find(|l| l.contains(task) && l.contains("over-constrained scope: "))
        .map(|l| {
            l.split("over-constrained scope: ")
                .nth(1)
                .and_then(|r| r.split_whitespace().next())
                .and_then(|n| n.parse().ok())
                .expect("over-constrained scope: <n> characters")
        })
}

#[test]
fn the_over_constrained_charge_stops_counting_an_enforced_rule() {
    let r = Repo::new("enforced-by-charge");
    let held = r.adr("Every module logs", HELD_RULE, &[]);
    r.accept(&held);
    let loose = r.adr("Every module tests", LOOSE_RULE, &[]);
    r.accept(&loose);
    let task = r.task("Some work under src");
    r.commit("task");
    // A limit of 50, half the budget, which each rule alone exceeds.
    r.ok(&["config", "context_budget", "100"]);
    r.commit("budget");

    let before = r.check();
    let held_chars = HELD_RULE.chars().count() + 1;
    let loose_chars = LOOSE_RULE.chars().count() + 1;
    assert_eq!(charge(&before, &held), Some(held_chars), "{before}");
    assert_eq!(charge(&before, &loose), Some(loose_chars), "{before}");
    assert_eq!(
        weight(&before, &task),
        Some(held_chars + loose_chars),
        "{before}"
    );

    r.ok(&["amend", &held, "--enforced-by", HELD]);
    r.commit("enforce");
    let after = r.check();
    assert_eq!(charge(&after, &held), None, "{after}");
    assert_eq!(charge(&after, &loose), Some(loose_chars), "{after}");
    assert_eq!(weight(&after, &task), Some(loose_chars), "{after}");
}
