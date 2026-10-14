//! A verifier declares `inputs` and `env`, and its definition hash covers both
//! (ADR-d8a328e311b9, TASK-dc04757dfecb).
//!
//! **Through the binary**, because every clause of the criterion is about it:
//! the keys are written and read back by `ank config`, the image tag is refused
//! by the parser every verb loads the file through, the hash is the one `done`
//! records in a proof, and the change is the one `check` reports against it.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A digest of the right shape. Nothing pulls it: ank parses, validates and
/// hashes an image, and never runs one (ADR-d8a328e311b9, point 5).
const IMAGE: &str =
    "ghcr.io/example/rust@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// The hash of `run: "true"` at the default timeout, as `done` recorded it
/// before `inputs` and `env` existed: `sha256("true\0600")`, first twelve
/// hex digits. A verifier declaring neither must keep it, or every proof in
/// every corpus would read as a changed verifier the day this lands.
const HASH_OF_TRUE: &str = "727d92e1ed4b";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("verifier-env-gitconfig");
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
        .env("ANK_AGENT", "test/verifier-env")
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

/// A corpus with one commit and one verifier, `t`, running `true`.
fn corpus(what: &str) -> PathBuf {
    let dir = scratch::dir(what);
    run("git", &dir, &["init", "-q", "-b", "main"]);
    fs::write(dir.join("f"), "x\n").unwrap();
    run(ANK, &dir, &["init"]);
    run(ANK, &dir, &["config", "default_branch", "main"]);
    run(ANK, &dir, &["config", "verifiers.t.run", "true"]);
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "init"]);
    dir
}

fn config_yml(dir: &Path) -> String {
    fs::read_to_string(dir.join(".ank/config.yml")).unwrap()
}

#[test]
fn inputs_and_every_env_key_are_written_to_config_yml_and_read_back() {
    let dir = corpus("verifier-env-roundtrip");

    let writes = [
        ("verifiers.t.inputs", r#"["crates/auth/**", "Cargo.toml"]"#),
        ("verifiers.t.env.platform", "linux/amd64"),
        ("verifiers.t.env.image", IMAGE),
        ("verifiers.t.env.vars.RUSTFLAGS", "-D warnings"),
        ("verifiers.t.env.network", "open"),
    ];
    for (key, value) in writes {
        // `--` because a value may open on a dash, as `-D warnings` does.
        run(ANK, &dir, &["config", "--", key, value]);
    }
    for (key, value) in writes {
        assert_eq!(run(ANK, &dir, &["config", key]), value, "reading {key}");
    }

    let text = config_yml(&dir);
    let block = format!(
        "  t:\n    run: \"true\"\n    inputs: [\"crates/auth/**\", \"Cargo.toml\"]\n    env:\n      \
         platform: linux/amd64\n      image: {IMAGE}\n      vars:\n        RUSTFLAGS: \"-D warnings\"\n      \
         network: open\n"
    );
    assert!(text.contains(&block), "config.yml:\n{text}");

    // The file every other verb loads still loads.
    run(ANK, &dir, &["check"]);

    // `network` absent under an image is `none`, and the reader is told it is
    // the tool's value and not the file's.
    run(ANK, &dir, &["config", "--unset", "verifiers.t.env.network"]);
    assert_eq!(
        run(ANK, &dir, &["config", "verifiers.t.env.network"]),
        "none (default)"
    );

    // The last variable removed takes `vars:` with it, and the last env key
    // takes `env:`: an empty mapping left behind is a key nobody wrote.
    run(
        ANK,
        &dir,
        &["config", "--unset", "verifiers.t.env.vars.RUSTFLAGS"],
    );
    assert!(!config_yml(&dir).contains("vars:"), "{}", config_yml(&dir));
    run(ANK, &dir, &["config", "--unset", "verifiers.t.env"]);
    run(ANK, &dir, &["config", "--unset", "verifiers.t.inputs"]);
    let text = config_yml(&dir);
    assert!(text.contains("  t:\n    run: \"true\"\n"), "{text}");
    assert!(
        !text.contains("env:") && !text.contains("inputs:"),
        "{text}"
    );
}

#[test]
fn an_image_named_by_tag_is_refused_when_the_file_is_parsed() {
    let dir = corpus("verifier-env-tag");
    run(
        ANK,
        &dir,
        &["config", "verifiers.t.env.platform", "linux/amd64"],
    );
    let before = config_yml(&dir);

    // Through the writer: the write would leave the file unreadable, so it is
    // refused and nothing moves.
    let out = spawn(
        ANK,
        &dir,
        &[
            "config",
            "verifiers.t.env.image",
            "ghcr.io/example/rust:1.80",
        ],
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a tag was written: {err}");
    assert!(err.contains("sha256"), "{err}");
    assert_eq!(config_yml(&dir), before);

    // By hand: any verb that loads the file refuses it, and says why.
    fs::write(
        dir.join(".ank/config.yml"),
        before.replace(
            "      platform: linux/amd64\n",
            "      platform: linux/amd64\n      image: ghcr.io/example/rust:1.80\n",
        ),
    )
    .unwrap();
    let out = spawn(ANK, &dir, &["check"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "check loaded a tagged image");
    assert!(
        err.contains("verifiers.t.env.image") && err.contains("tag"),
        "{err}"
    );
}

/// `done` records `t@<hash>`, and `check` compares it with the definition in
/// force: changing `inputs` or `env` is a verifier change like any other.
#[test]
fn the_definition_hash_covers_inputs_and_env_and_check_reports_their_change() {
    let dir = corpus("verifier-env-hash");
    let new = run(
        ANK,
        &dir,
        &[
            "new",
            "task",
            "--title",
            "closed by t",
            "--scope",
            "f",
            "--criteria",
            "t passes",
            "--verify",
            "t",
            "--json",
        ],
    );
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "new"]);
    run(ANK, &dir, &["claim", &id]);
    run(ANK, &dir, &["done"]);
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "done"]);

    let shown = run(ANK, &dir, &["show", &id]);
    assert!(
        shown.contains(&format!("t@{HASH_OF_TRUE}")),
        "a verifier declaring neither keeps its hash:\n{shown}"
    );
    let changed = format!("verifier 't' changed since the proof ({HASH_OF_TRUE} -> ");
    assert!(!run(ANK, &dir, &["check"]).contains(&changed));

    for (key, value) in [
        ("verifiers.t.inputs", r#"["f"]"#),
        ("verifiers.t.env.vars.LANG", "C"),
        ("verifiers.t.env.platform", "linux/amd64"),
    ] {
        run(ANK, &dir, &["config", key, value]);
        let report = run(ANK, &dir, &["check"]);
        assert!(report.contains(&changed), "after {key}:\n{report}");
        run(ANK, &dir, &["config", "--unset", key]);
        let report = run(ANK, &dir, &["check"]);
        assert!(!report.contains(&changed), "after --unset {key}:\n{report}");
    }
}
