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

    /// `accept`, then the ratification committed as the anchor the commit
    /// walk reads (`crate::legacy_anchor`) and signed under a committer date
    /// the test chooses: the message and the tree are the ones `accept`
    /// wrote, so the anchor still reads, and only the date and the signature
    /// move.
    fn ratify_at(&self, id: &str, committed: &str) {
        let out = self.ank(&["accept", id]);
        assert_eq!(out.status.code(), Some(0), "accept: {}", text(&out.stderr));
        crate::legacy_anchor::commit(&|a| git(&self.repo, a), id, false);
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

// ---------------------------------------------------------------------------
// `ank signers add`, `retire` and `remove` (TASK-68313ec97cd6)
// ---------------------------------------------------------------------------

impl Corpus {
    fn ok(&self, args: &[&str]) {
        let out = self.ank(args);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
    }

    /// A refused call, its code, and the proof that it wrote nothing.
    fn refused(&self, args: &[&str], code: i32) -> String {
        let before = self.config();
        let out = self.ank(args);
        assert_eq!(
            out.status.code(),
            Some(code),
            "{args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        assert_eq!(self.config(), before, "{args:?}: a refusal writes nothing");
        text(&out.stderr)
    }

    fn commit_config(&self) {
        git(&self.repo, &["commit", "-qam", "signers"]);
    }
}

/// Today's date in UTC, as `retire` writes it when no `--date` is given.
fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let z = secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `add` with no flag declares the key `user.signingkey` names under the
/// principal `user.email` names, appending the block below every byte that
/// was there; a second `add` of that key is refused at 6 and writes nothing.
#[test]
fn add_declares_the_signing_key_under_user_email_and_refuses_it_twice() {
    let c = Corpus::new("signers-add-ssh");
    let (key, declared) = c.ssh_key("laptop");
    c.sign_with_ssh(&key);
    git(&c.repo, &["config", "user.email", "marie@laptop"]);
    let before = c.config();

    c.ok(&["signers", "add"]);
    assert_eq!(
        c.config(),
        format!("{before}signers:\n  marie@laptop:\n    keys:\n      - key: {declared}\n"),
    );

    let after = c.config();
    let said = c.refused(&["signers", "add", "--note", "again"], 6);
    assert!(said.contains("already declared"), "{said}");
    assert_eq!(c.config(), after);
}

/// Under OpenPGP, `user.signingkey` is the fingerprint, declared as `gpg`.
#[test]
fn add_declares_a_gpg_signing_key_by_its_fingerprint() {
    let c = Corpus::new("signers-add-gpg");
    c.sign_with_gpg();
    git(&c.repo, &["config", "user.email", "marie@laptop"]);
    let before = c.config();
    c.ok(&["signers", "add", "--note", "release key"]);
    assert_eq!(
        c.config(),
        format!(
            "{before}signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n        note: release key\n"
        ),
    );
}

/// `--key`, `--principal` and `--note` override what git names, and a key is
/// written beside its siblings in the indentation they already use: under its
/// principal after the last key there, or as a new principal after the last
/// one. Comments and order elsewhere are what they were, byte for byte.
#[test]
fn add_writes_only_its_lines_into_a_block_written_by_hand() {
    let c = Corpus::new("signers-add-surgical");
    let (_, laptop) = c.ssh_key("laptop");
    let (vps, vps_pub) = c.ssh_key("vps");
    let (_, desk_pub) = c.ssh_key("desk");
    let hand = format!(
        "schema: 1\n\
         # Who may ratify, reviewed like code.\n\
         signers:\n\
         \x20 # The maintainer.\n\
         \x20 \"marie@laptop\":\n\
         \x20   keys:\n\
         \x20   - key: gpg {FPR}   # release\n\
         \x20     note: 'release key'\n\
         \x20   # The laptop below the comment.\n\
         \x20   - key: {laptop}\n\
         \n\
         # What follows the block introduces the next key.\n\
         default_branch: main\n"
    );
    fs::write(c.repo.join(".ank/config.yml"), &hand).unwrap();
    c.commit_config();

    // A new key of a principal that exists: after its last key, at its
    // siblings' indentation, above the blank line and the comment.
    c.ok(&[
        "signers",
        "add",
        "--key",
        vps.with_extension("pub").to_str().unwrap(),
        "--principal",
        "marie@laptop",
        "--note",
        "VPS: the one in Paris",
    ]);
    let with_vps = hand.replace(
        &format!("    - key: {laptop}\n"),
        &format!(
            "    - key: {laptop}\n    - key: {vps_pub}\n      note: \"VPS: the one in Paris\"\n"
        ),
    );
    assert_eq!(c.config(), with_vps);

    // A principal that does not exist: after the last one, at its indentation.
    c.ok(&[
        "signers",
        "add",
        "--key",
        &desk_pub,
        "--principal",
        "bob@desk",
    ]);
    let with_bob = with_vps.replace(
        "\n\n# What follows",
        &format!("\n  bob@desk:\n    keys:\n      - key: {desk_pub}\n\n# What follows"),
    );
    assert_eq!(c.config(), with_bob);

    // Declared under another principal is still declared.
    let said = c.refused(
        &[
            "signers",
            "add",
            "--key",
            &desk_pub,
            "--principal",
            "marie@laptop",
        ],
        6,
    );
    assert!(said.contains("bob@desk"), "names who declares it: {said}");
}

/// What `add` cannot resolve is refused at 7, naming the flag that supplies it.
#[test]
fn add_without_a_key_or_a_principal_names_the_flag_and_writes_nothing() {
    let c = Corpus::new("signers-add-missing");
    git(&c.repo, &["config", "user.email", "marie@laptop"]);
    let said = c.refused(&["signers", "add"], 7);
    assert!(said.contains("--key"), "{said}");

    let (_, declared) = c.ssh_key("laptop");
    git(&c.repo, &["config", "--unset", "user.email"]);
    let out = spawn(ANK)
        .args(["signers", "add", "--key", &declared])
        .env("ANK_AGENT", HUMAN)
        .env("GIT_CONFIG_GLOBAL", c.repo.join("keys/none"))
        .env("GIT_CONFIG_SYSTEM", c.repo.join("keys/none"))
        .current_dir(&c.repo)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(7), "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains("--principal"),
        "{}",
        text(&out.stderr)
    );
    assert!(!c.config().contains("signers:"));
}

/// `retire` sets `valid_before`: what the key signed before the date stays
/// clean in `check`, what it signed after is a fault. Without `--date` it is
/// today; a second `retire` moves the date in place rather than adding a line.
#[test]
fn retire_sets_valid_before_and_check_judges_by_it() {
    let c = Corpus::new("signers-retire");
    let (key, declared) = c.ssh_key("laptop");
    c.sign_with_ssh(&key);
    c.ok(&[
        "signers",
        "add",
        "--principal",
        "marie@laptop",
        "--note",
        "laptop",
    ]);
    let added = c.config();
    c.commit_config();
    c.ratify_at(A, BEFORE);

    c.ok(&["signers", "retire", &declared, "--date", "2026-10-01"]);
    assert_eq!(
        c.config(),
        format!("{added}        valid_before: 2026-10-01\n"),
        "one line added under the key, nothing else moved"
    );
    c.commit_config();
    c.ratify_at(B, AFTER);

    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(
        faults_about(&report, A).is_empty(),
        "signed before: {report}"
    );
    assert!(
        faults_about(&report, B)
            .iter()
            .any(|l| l.contains("valid_before") && l.contains("2026-10-01")),
        "signed after: {report}"
    );

    // The key alone names it as well as `<keytype> <key>` does.
    let bare = declared.split_whitespace().nth(1).unwrap();
    c.ok(&["signers", "retire", bare]);
    assert_eq!(
        c.config(),
        format!("{added}        valid_before: {}\n", today()),
        "the date moved in place"
    );

    c.refused(&["signers", "retire", bare, "--date", "2026-13-01"], 1);
    let said = c.refused(&["signers", "retire", "ssh-ed25519 AAAAnotdeclared"], 1);
    assert!(said.contains("declares no key"), "{said}");
    c.refused(&["signers", "retire"], 7);
}

/// `remove` deletes the key, and every ratification it signed becomes a
/// fault. The last key of a principal takes the principal with it, and the
/// last principal the block: what is left is the file as it was before.
///
/// Another principal stays declared throughout, because a corpus declaring no
/// key at all is advisory (§8) and faults nothing.
#[test]
fn remove_deletes_the_key_and_what_it_ratified_becomes_a_fault() {
    let c = Corpus::new("signers-remove");
    let (key, declared) = c.ssh_key("laptop");
    let (_, other) = c.ssh_key("vps");
    c.sign_with_ssh(&key);
    let empty = c.config();
    let gpg = format!("gpg {FPR}");
    c.ok(&["signers", "add", "--principal", "bob@desk", "--key", &gpg]);
    let bob = c.config();
    c.ok(&["signers", "add", "--principal", "marie@laptop"]);
    let one = c.config();
    c.ok(&[
        "signers",
        "add",
        "--principal",
        "marie@laptop",
        "--key",
        &other,
        "--note",
        "vps",
    ]);
    c.commit_config();
    c.ratify_at(A, BEFORE);
    c.ratify_at(B, AFTER);
    let (_, report) = c.check();
    assert!(faults_about(&report, A).is_empty(), "{report}");
    assert!(faults_about(&report, B).is_empty(), "{report}");

    c.ok(&["signers", "remove", &other]);
    assert_eq!(
        c.config(),
        one,
        "the vps key and its note, and nothing else"
    );

    c.ok(&["signers", "remove", &declared]);
    assert_eq!(c.config(), bob, "the last key takes its principal with it");
    c.commit_config();

    let (code, report) = c.check();
    assert_eq!(code, Some(8), "{report}");
    assert!(!faults_about(&report, A).is_empty(), "{report}");
    assert!(!faults_about(&report, B).is_empty(), "{report}");

    c.refused(&["signers", "remove", &declared], 1);
    c.ok(&["signers", "remove", FPR]);
    assert_eq!(
        c.config(),
        empty,
        "the last principal takes the block with it"
    );
}

/// `ank help signers` and the contract declare the three subcommands, their
/// flags and their refusals.
#[test]
fn help_declares_add_retire_and_remove_and_their_refusals() {
    let c = Corpus::new("signers-help");
    let out = c.ank(&["help", "signers"]);
    let page = text(&out.stdout);
    for word in [
        "add",
        "retire",
        "remove",
        "migrate",
        "--key",
        "--principal",
        "--note",
        "--date",
        "already declared",
        "(6)",
    ] {
        assert!(page.contains(word), "{word}: {page}");
    }
    let out = c.ank(&["help", "--json"]);
    let json = text(&out.stdout);
    let at = json.find(r#"{"name":"signers","usage""#).expect(&json);
    let verb = &json[at..];
    let verb = &verb[..verb.find(r#"{"name":"init","usage""#).unwrap_or(verb.len())];
    assert!(
        verb.contains(r#""usage":"ank signers <add|retire|remove|migrate> [<key>]""#),
        "{verb}"
    );
    assert!(verb.contains(r#""code":6"#), "{verb}");
}
