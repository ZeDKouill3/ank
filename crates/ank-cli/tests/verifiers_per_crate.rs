//! One default test verifier per workspace crate, each declaring what it reads
//! (TASK-e9f5c4e8922a).
//!
//! **Why the graph is read from cargo and not typed here.** A verifier whose
//! `inputs` omit a crate its tests link is the false reuse ADR-a2faf59f6ce0
//! names: `done` would reuse a green run of `ank-tui` after a change to
//! `ank-contract` it was never shown. The list of what each crate depends on is
//! `cargo metadata`'s to answer, so a crate that gains a dependency turns this
//! red the day it does, rather than the day somebody remembers.
//!
//! **Why `inputs` are confronted with files and not compared as strings.** What
//! `done` keys a run on is the files a glob matches (`verify.rs`), so the
//! question is whether every file of a crate is matched, whatever the glob
//! looks like. `ank-cli` declares `**` because its suite reads the corpus, the
//! docs, the skills and every manifest of the tree, and that answers the
//! question as well as a list would.

use crate::scratch;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// A workspace crate: its name, its directory relative to the root, and the
/// workspace crates it depends on, of any kind.
#[derive(Debug)]
struct Crate {
    name: String,
    dir: String,
    deps: BTreeSet<String>,
}

/// `cargo metadata --no-deps`, read with the YAML parser this crate already
/// carries: JSON is YAML, and no dependency enters for one test.
fn workspace(root: &Path) -> Vec<Crate> {
    let out = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .expect("cargo metadata must run");
    assert!(
        out.status.success(),
        "cargo metadata: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect("metadata is JSON");
    let packages = doc["packages"]
        .as_sequence()
        .expect("metadata lists packages");
    let names: BTreeSet<String> = packages
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    packages
        .iter()
        .map(|p| {
            let manifest = PathBuf::from(p["manifest_path"].as_str().unwrap());
            let dir = manifest
                .parent()
                .unwrap()
                .canonicalize()
                .unwrap()
                .strip_prefix(root)
                .expect("a workspace crate sits under the root")
                .to_string_lossy()
                .replace('\\', "/");
            let deps = p["dependencies"]
                .as_sequence()
                .unwrap()
                .iter()
                .map(|d| d["name"].as_str().unwrap().to_string())
                .filter(|n| names.contains(n))
                .collect();
            Crate {
                name: p["name"].as_str().unwrap().to_string(),
                dir,
                deps,
            }
        })
        .collect()
}

/// Every file under `dir`, relative to `root`, `/`-separated.
fn files(root: &Path, dir: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.join(dir)];
    while let Some(at) = stack.pop() {
        for entry in fs::read_dir(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(root).unwrap().to_string_lossy();
                out.push(rel.replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

/// The verifiers a `config.yml` declares, by name.
fn verifiers(config: &str) -> BTreeMap<String, serde_yaml::Value> {
    let doc: serde_yaml::Value = serde_yaml::from_str(config).expect("config.yml is YAML");
    doc["verifiers"]
        .as_mapping()
        .expect("config.yml declares verifiers")
        .iter()
        .map(|(k, v)| (k.as_str().unwrap().to_string(), v.clone()))
        .collect()
}

fn inputs(verifier: &serde_yaml::Value) -> Vec<String> {
    verifier["inputs"]
        .as_sequence()
        .map(|s| s.iter().map(|g| g.as_str().unwrap().to_string()).collect())
        .unwrap_or_default()
}

/// What each crate's verifier fails to declare, one line per omission; empty
/// when every crate has a default verifier running its tests and reading its
/// crate, every workspace crate it reaches, `Cargo.toml` and `Cargo.lock`.
fn omissions(root: &Path, config: &str, crates: &[Crate]) -> Vec<String> {
    let declared = verifiers(config);
    let by_name: BTreeMap<&str, &Crate> = crates.iter().map(|c| (c.name.as_str(), c)).collect();
    let mut found = Vec::new();
    for c in crates {
        let name = format!("test-{}", c.name);
        let Some(v) = declared.get(&name) else {
            found.push(format!("{}: no verifier {name}", c.name));
            continue;
        };
        if v["default"].as_bool() != Some(true) {
            found.push(format!("{name}: not marked default: true"));
        }
        let run = v["run"].as_str().unwrap_or("");
        if !run.contains(&format!("-p {}", c.name)) {
            found.push(format!("{name}: run does not name -p {}: {run}", c.name));
        }
        let globs = inputs(v);
        if globs.is_empty() {
            found.push(format!("{name}: declares no inputs"));
            continue;
        }
        let set = ank_core::ScopeSet::new(&globs).expect("inputs compile");

        // The crate and its workspace dependencies, transitively: a crate
        // links what its dependencies link.
        let mut reach = BTreeSet::from([c.name.clone()]);
        let mut todo = vec![c.name.clone()];
        while let Some(n) = todo.pop() {
            for d in &by_name[n.as_str()].deps {
                if reach.insert(d.clone()) {
                    todo.push(d.clone());
                }
            }
        }
        for r in &reach {
            let dir = &by_name[r.as_str()].dir;
            if let Some(missed) = files(root, dir).into_iter().find(|f| !set.matches(f)) {
                found.push(format!("{name}: inputs omit {r} ({missed})"));
            }
        }
        for manifest in ["Cargo.toml", "Cargo.lock"] {
            if !set.matches(manifest) {
                found.push(format!("{name}: inputs omit {manifest}"));
            }
        }
    }
    found
}

fn config(root: &Path) -> String {
    fs::read_to_string(root.join(".ank/config.yml")).expect(".ank/config.yml is readable")
}

#[test]
fn every_workspace_crate_has_a_default_verifier_reading_what_it_depends_on() {
    let root = root();
    let crates = workspace(&root);
    assert!(
        crates.len() > 1,
        "the workspace has several crates: {crates:?}"
    );
    let found = omissions(&root, &config(&root), &crates);
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// The check above is worth something only if it turns red. Drop one
/// dependency's glob from a verifier that needs it, and it must say which.
#[test]
fn a_verifier_whose_inputs_omit_a_workspace_dependency_is_reported() {
    let root = root();
    let crates = workspace(&root);
    let (crate_, dep) = crates
        .iter()
        .find_map(|c| c.deps.iter().next().map(|d| (c, d)))
        .expect("some workspace crate depends on another");
    let name = format!("test-{}", crate_.name);
    let dep_dir = &crates.iter().find(|c| &c.name == dep).unwrap().dir;

    let mut doc: serde_yaml::Value = serde_yaml::from_str(&config(&root)).unwrap();
    let declared = &mut doc["verifiers"][name.as_str()];
    let kept: Vec<serde_yaml::Value> = crates
        .iter()
        .filter(|c| &c.name != dep)
        .map(|c| serde_yaml::Value::String(format!("{}/**", c.dir)))
        .chain(["Cargo.toml", "Cargo.lock"].map(|s| serde_yaml::Value::String(s.into())))
        .collect();
    declared["inputs"] = serde_yaml::Value::Sequence(kept);
    let edited = serde_yaml::to_string(&doc).unwrap();

    let found = omissions(&root, &edited, &crates);
    assert!(
        found
            .iter()
            .any(|f| f.starts_with(&format!("{name}: inputs omit {dep} ({dep_dir}/"))),
        "dropping {dep} from {name} must be reported: {found:?}"
    );
}

/// `fmt-check` reads every Rust source and the rustfmt configuration.
#[test]
fn fmt_check_declares_the_rust_sources_and_rustfmt_toml() {
    let root = root();
    let declared = verifiers(&config(&root));
    let globs = inputs(&declared["fmt-check"]);
    let set = ank_core::ScopeSet::new(&globs).expect("inputs compile");
    let sources: Vec<String> = files(&root, "crates")
        .into_iter()
        .filter(|f| f.ends_with(".rs"))
        .collect();
    assert!(!sources.is_empty());
    let missed: Vec<&String> = sources.iter().filter(|f| !set.matches(f)).collect();
    assert!(
        missed.is_empty(),
        "fmt-check inputs {globs:?} omit {missed:?}"
    );
    for config in ["rustfmt.toml", ".rustfmt.toml"] {
        assert!(
            set.matches(config),
            "fmt-check inputs {globs:?} omit {config}"
        );
    }
}

fn run(dir: &Path, program: &str, args: &[&str]) -> Output {
    let home = scratch::root();
    let gitconfig = scratch::path("verifiers-per-crate-gitconfig");
    fs::write(
        &gitconfig,
        "[commit]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
    )
    .unwrap();
    let out = Command::new(program)
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", &gitconfig)
        .env("GIT_CONFIG_SYSTEM", &gitconfig)
        .env("ANK_AGENT", "test/verifiers-per-crate")
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

/// Through the binary: a task written here with no flag carries the per-crate
/// verifiers and `fmt-check`, and not the whole-workspace `cargo-test`.
#[test]
fn ank_new_task_with_no_flag_lands_with_the_per_crate_verifiers() {
    let root = root();
    let crates = workspace(&root);
    let repo = scratch::dir("verifiers-per-crate");
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    run(&repo, ANK, &["init"]);
    fs::write(repo.join(".ank/config.yml"), config(&root)).unwrap();

    let out = run(
        &repo,
        ANK,
        &[
            "new",
            "task",
            "--title",
            "A task",
            "--scope",
            "src/**",
            "--criteria",
            "x",
            "--json",
        ],
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let at = text.find("TASK-").expect("new task --json names the id");
    let id = &text[at..at + "TASK-".len() + 12];
    let shown = run(&repo, ANK, &["show", id]);
    let shown = String::from_utf8_lossy(&shown.stdout);
    let line = shown
        .lines()
        .find(|l| l.starts_with("verify:"))
        .unwrap_or_else(|| panic!("the task declares verify: {shown}"));
    let verify: BTreeSet<String> = line["verify:".len()..]
        .trim()
        .trim_matches(['[', ']'])
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut expected: BTreeSet<String> =
        crates.iter().map(|c| format!("test-{}", c.name)).collect();
    expected.insert("fmt-check".to_string());
    assert_eq!(verify, expected, "{shown}");
}
