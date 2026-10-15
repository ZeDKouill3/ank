//! Who may ratify lives in `config.yml` under `signers:` (ADR-5c6db7a161b8,
//! TASK-3a7a73ccf2b7).
//!
//! Through the binary throughout: `check` reads `signers:` and nothing else,
//! a key retired by `valid_before` ratifies nothing committed after the date,
//! `ank signers migrate` moves a former `.ank/allowed_signers` into the block,
//! and `config` refuses to write it.
//!
//! **Signed for real, under both formats.** SSH through `ssh-keygen`, which
//! ships beside git on all three platforms. OpenPGP through a stand-in
//! `gpg.program` compiled once per run, because a runner is not guaranteed a
//! gpg and a keyring generated per test would cost seconds each: git speaks to
//! gpg through `--status-fd` lines, and the stand-in answers in exactly those,
//! naming one fixed fingerprint. What is under test is what `check` does with
//! git's answer, and git is real.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const HUMAN: &str = "human:marie";
const A: &str = "ADR-00000000005a";
const B: &str = "ADR-00000000005b";
const EXE: &str = std::env::consts::EXE_SUFFIX;

/// The fingerprint the stand-in gpg signs and verifies as.
const FPR: &str = "0123456789ABCDEF0123456789ABCDEF01234567";

/// Before `valid_before`, and after it.
const BEFORE: &str = "2026-09-15T12:00:00+00:00";
const AFTER: &str = "2026-10-11T12:00:00+00:00";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("signers-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [init]\n\tdefaultBranch = main\n",
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn git_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut c = spawn("git");
    c.args(args).current_dir(dir);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().expect("git must be on PATH");
    assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    text(&out.stdout).trim().to_string()
}

fn git(dir: &Path, args: &[&str]) -> String {
    git_env(dir, args, &[])
}

/// The stand-in gpg. Signing prints an armoured block and the `SIG_CREATED`
/// line git looks for on the status descriptor; verifying prints the
/// `GOODSIG`/`VALIDSIG` pair git reads `%G?` and `%GF` from. Both read their
/// input to the end, so git never writes into a closed pipe.
const GPG_STUB: &str = r#"
use std::io::Read;
const FPR: &str = "0123456789ABCDEF0123456789ABCDEF01234567";
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    if args.iter().any(|a| a == "--verify") {
        print!(
            "[GNUPG:] NEWSIG\n[GNUPG:] GOODSIG {} Marie <marie@example.com>\n\
             [GNUPG:] VALIDSIG {FPR} 2026-01-01 1767225600 0 4 0 22 8 00 {FPR}\n\
             [GNUPG:] TRUST_ULTIMATE 0 pgp\n",
            &FPR[24..]
        );
    } else {
        print!("-----BEGIN PGP SIGNATURE-----\n\niQEzBAABCAAdFiEEASNFZ4mrze8=\n=abcd\n-----END PGP SIGNATURE-----\n");
        eprint!("\n[GNUPG:] SIG_CREATED D 22 8 00 1767225600 {FPR}\n");
    }
}
"#;

fn gpg_stub() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let src = scratch::path("signers-gpg-stub.rs");
        let exe = scratch::path(&format!("signers-gpg-stub{EXE}"));
        fs::write(&src, GPG_STUB).unwrap();
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

fn adr(id: &str) -> String {
    format!(
        "---\nid: {id}\ntype: adr\nslug: a-decision\ntitle: A decision\n\
         created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
         scope:\n  - src/**\nconstraint: |\n  A binding rule.\nschema: 3\nversion: 1\n---\n\nWhy.\n"
    )
}

/// The head of `config.yml`, with a comment in it, so a write that rewrote
/// anything above `signers:` would show.
const CONFIG_HEAD: &str =
    "schema: 1\n# Reviewed like code.\nclaim_ttl_max: 2h   # a lease\ndefault_branch: main\n";

struct Corpus {
    repo: PathBuf,
}

impl Corpus {
    /// Two proposed ADRs on `main`, nothing declared yet.
    fn new(what: &str) -> Corpus {
        let repo = scratch::dir(what);
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(repo.join(".ank/config.yml"), CONFIG_HEAD).unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db*\nkeys/\n").unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        for id in [A, B] {
            fs::write(repo.join(".ank/entities").join(format!("{id}.md")), adr(id)).unwrap();
        }
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);
        Corpus { repo }
    }

    /// A fresh ssh key, its public half as `config.yml` declares one:
    /// `<keytype> <base64>`, the comment left out.
    fn ssh_key(&self, name: &str) -> (PathBuf, String) {
        let dir = self.repo.join("keys");
        fs::create_dir_all(&dir).unwrap();
        let key = dir.join(name);
        let out = spawn("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-C", "marie@laptop", "-q", "-f"])
            .arg(&key)
            .output()
            .expect("ssh-keygen ships with git on all three platforms");
        assert!(out.status.success(), "ssh-keygen: {}", text(&out.stderr));
        let public = fs::read_to_string(key.with_extension("pub")).unwrap();
        let declared: Vec<&str> = public.split_whitespace().take(2).collect();
        (key, declared.join(" "))
    }

    fn sign_with_ssh(&self, key: &Path) {
        git(&self.repo, &["config", "gpg.format", "ssh"]);
        git(
            &self.repo,
            &[
                "config",
                "user.signingkey",
                key.with_extension("pub").to_str().unwrap(),
            ],
        );
    }

    fn sign_with_gpg(&self) {
        git(&self.repo, &["config", "gpg.format", "openpgp"]);
        git(
            &self.repo,
            &["config", "gpg.program", gpg_stub().to_str().unwrap()],
        );
        git(&self.repo, &["config", "user.signingkey", FPR]);
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .env("ANK_AGENT", HUMAN)
            .current_dir(&self.repo)
            .output()
            .expect("the binary must have been built")
    }

    /// `accept`, then the ratification re-signed under a committer date the
    /// test chooses: the message and the tree are the ones `accept` wrote, so
    /// the anchor still reads, and only the date and the signature move.
    fn ratify_at(&self, id: &str, committed: &str) {
        let out = self.ank(&["accept", id]);
        assert_eq!(out.status.code(), Some(0), "accept: {}", text(&out.stderr));
        git_env(
            &self.repo,
            &["commit", "--amend", "--no-edit", "-q", "-S"],
            &[("GIT_COMMITTER_DATE", committed)],
        );
    }

    fn config(&self) -> String {
        fs::read_to_string(self.repo.join(".ank/config.yml")).unwrap()
    }

    fn append_config(&self, tail: &str) {
        let mut text = self.config();
        text.push_str(tail);
        fs::write(self.repo.join(".ank/config.yml"), text).unwrap();
        git(&self.repo, &["commit", "-qam", "declare"]);
    }

    fn check(&self) -> (Option<i32>, String) {
        let out = self.ank(&["check"]);
        (out.status.code(), text(&out.stdout) + &text(&out.stderr))
    }
}

/// The fault lines `check` printed about one entity.
fn faults_about(report: &str, id: &str) -> Vec<String> {
    report
        .lines()
        .filter(|l| l.starts_with("error:") && l.contains(id))
        .map(str::to_string)
        .collect()
}

/// A retired ssh key: what it signed before `valid_before` stays verified, and
/// what it signed after is a fault.
#[test]
fn an_ssh_key_ratifies_nothing_committed_after_its_valid_before() {
    let c = Corpus::new("signers-ssh-valid-before");
    let (key, declared) = c.ssh_key("laptop");
    c.sign_with_ssh(&key);
    c.append_config(&format!(
        "signers:\n  marie@laptop:\n    keys:\n      - key: {declared}\n        valid_before: 2026-10-01\n"
    ));
    c.ratify_at(A, BEFORE);
    c.ratify_at(B, AFTER);

    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        faults_about(&report, A).is_empty(),
        "signed before valid_before, so still verified: {report}"
    );
    let b = faults_about(&report, B);
    assert!(
        b.iter()
            .any(|l| l.contains("valid_before") && l.contains("2026-10-01")),
        "signed after valid_before is a fault naming the date: {report}"
    );
}

/// The same under OpenPGP, where git never reads an allowed-signers file and
/// `check` judges the committer date itself.
#[test]
fn a_gpg_key_ratifies_nothing_committed_after_its_valid_before() {
    let c = Corpus::new("signers-gpg-valid-before");
    c.sign_with_gpg();
    c.append_config(&format!(
        "signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n        valid_before: 2026-10-01\n"
    ));
    c.ratify_at(A, BEFORE);
    c.ratify_at(B, AFTER);

    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        faults_about(&report, A).is_empty(),
        "signed before valid_before, so still verified: {report}"
    );
    let b = faults_about(&report, B);
    assert!(
        b.iter()
            .any(|l| l.contains("valid_before") && l.contains("2026-10-01")),
        "signed after valid_before is a fault naming the date: {report}"
    );
}

/// The control the two above rest on: with no `valid_before`, both
/// ratifications are clean, so the fault is the date and nothing else.
#[test]
fn a_key_without_valid_before_verifies_both_ratifications() {
    let c = Corpus::new("signers-no-valid-before");
    c.sign_with_gpg();
    c.append_config(&format!(
        "signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n        note: release key\n"
    ));
    c.ratify_at(A, BEFORE);
    c.ratify_at(B, AFTER);
    let (code, report) = c.check();
    assert_eq!(code, Some(0), "{report}");
    assert!(!report.contains("error:"), "{report}");
}

/// `ank signers migrate`: the file's keys land under `signers:`, keyed by
/// principal, each comment kept as a note; the file is gone; every byte of
/// `config.yml` above the block is what it was. Then `check` judges the
/// ratifications as the file did: one signed by a declared key is clean, one
/// signed by a key nobody declared is a fault.
#[test]
fn migrate_moves_allowed_signers_under_signers_and_check_judges_the_same() {
    let c = Corpus::new("signers-migrate");
    let (laptop, laptop_pub) = c.ssh_key("laptop");
    let (_vps, vps_pub) = c.ssh_key("vps");
    let (stranger, _) = c.ssh_key("stranger");
    let file = format!(
        "# Who may ratify.\n#\n# The gpg key is a declaration.\n\
         marie@laptop gpg {FPR}\n\
         # The laptop key.\n\
         marie@laptop {laptop_pub} marie@laptop\n\
         \n\
         # A key nobody signs with.\n\
         bob@vps {vps_pub}\n"
    );
    fs::write(c.repo.join(".ank/allowed_signers"), &file).unwrap();
    git(&c.repo, &["add", "-A"]);
    git(&c.repo, &["commit", "-qm", "declare"]);

    c.sign_with_ssh(&laptop);
    c.ratify_at(A, BEFORE);
    c.sign_with_ssh(&stranger);
    c.ratify_at(B, BEFORE);

    // Before: the file is still there, which is a fault naming the command.
    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        report
            .lines()
            .any(|l| l.starts_with("error:") && l.contains("ank signers migrate")),
        "a remaining allowed_signers is a fault naming the migration: {report}"
    );

    let before = c.config();
    let out = c.ank(&["signers", "migrate"]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(
        !c.repo.join(".ank/allowed_signers").exists(),
        "the file is deleted"
    );
    let after = c.config();
    assert!(
        after.starts_with(&before),
        "every byte above the block is unchanged:\n{after}"
    );
    assert_eq!(
        &after[before.len()..],
        format!(
            "signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n        note: |-\n          Who may ratify.\n\n          The gpg key is a declaration.\n      - key: {laptop_pub}\n        note: |-\n          The laptop key.\n          marie@laptop\n  bob@vps:\n    keys:\n      - key: {vps_pub}\n        note: A key nobody signs with.\n"
        ),
        "the block, as written"
    );

    git(&c.repo, &["add", "-A"]);
    git(&c.repo, &["commit", "-qm", "migrate"]);
    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        !report.contains("allowed_signers"),
        "nothing left to migrate: {report}"
    );
    assert!(
        faults_about(&report, A).is_empty(),
        "signed by a declared key: {report}"
    );
    assert!(
        faults_about(&report, B)
            .iter()
            .any(|l| l.contains("does not declare")),
        "signed by a key nobody declared: {report}"
    );

    // A second run has nothing to move, and says so rather than writing.
    let out = c.ank(&["signers", "migrate"]);
    assert_ne!(out.status.code(), Some(0), "{}", text(&out.stdout));
    assert_eq!(c.config(), after, "a refused run writes nothing");
}

/// `signers:` beside a remaining `.ank/allowed_signers` is a fault naming the
/// migration, and the file is not read: two truths about who may ratify are
/// refused, not reconciled.
#[test]
fn allowed_signers_beside_signers_is_a_fault_naming_the_migration() {
    let c = Corpus::new("signers-beside");
    let (key, declared) = c.ssh_key("laptop");
    c.append_config(&format!(
        "signers:\n  marie@laptop:\n    keys:\n      - key: {declared}\n"
    ));
    fs::write(
        c.repo.join(".ank/allowed_signers"),
        format!("marie@laptop {declared}\n"),
    )
    .unwrap();
    c.sign_with_ssh(&key);
    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        report.lines().any(|l| l.starts_with("error:")
            && l.contains("allowed_signers")
            && l.contains("ank signers migrate")),
        "{report}"
    );
}

/// `ank config` does not write who may ratify: that is `ank signers`.
#[test]
fn config_refuses_signers_and_names_ank_signers() {
    let c = Corpus::new("signers-config");
    let before = c.config();
    for args in [
        &["config", "signers.marie@laptop", "x"][..],
        &["config", "signers"][..],
        &["config", "--unset", "signers.marie@laptop"][..],
    ] {
        let out = c.ank(args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{args:?}: {}",
            text(&out.stdout)
        );
        assert!(
            text(&out.stderr).contains("ank signers"),
            "{args:?}: {}",
            text(&out.stderr)
        );
    }
    assert_eq!(c.config(), before, "nothing written");
}

/// `review` lists the principals `signers:` declares, one row per key.
#[test]
fn review_lists_the_principals_signers_declares() {
    let c = Corpus::new("signers-review");
    let (_, declared) = c.ssh_key("laptop");
    c.append_config(&format!(
        "signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n  bob@vps:\n    keys:\n      - key: {declared}\n        valid_before: 2026-10-01\n"
    ));
    let out = c.ank(&["review"]);
    let said = text(&out.stdout);
    assert!(said.contains("MAY RATIFY (2)"), "{said}");
    assert!(said.contains("marie@laptop  gpg"), "{said}");
    assert!(said.contains("bob@vps  ssh-ed25519"), "{said}");

    let out = c.ank(&["review", "--json"]);
    let json = text(&out.stdout);
    assert!(
        json.contains(r#"{"principal":"bob@vps","keytype":"ssh-ed25519"}"#)
            && json.contains(r#"{"principal":"marie@laptop","keytype":"gpg"}"#),
        "{json}"
    );
}
