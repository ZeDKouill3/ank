//! The claims panel, through the binary, on a real terminal
//! (TASK-574432893e0f, ADR-ac6be1ebe9aa).
//!
//! *Every live claim the reader can see is a row carrying the task, its
//! holder, how long ago it was claimed, and when it expires or that it has
//! lapsed, with the reader's own claims marked.* And *a search narrows the
//! focused listing as it is typed*, which the claims panel is as much as the
//! entities are.
//!
//! Two holders, so that "marked" has something to be told apart from: the
//! suite's own identity claims the seeded task, and a second identity claims a
//! second one. The clock is forged rather than waited for where the claim is
//! about time -- a lapse is judged with two minutes of drift tolerance on top
//! of the lease, and a claim taken years ago cannot be waited for at all -- by
//! rewriting one field of the record the binary wrote and leaving every other
//! byte alone, which is the surgery `crates/ank-cli/tests/cli.rs` already does.

#![cfg(unix)]

use std::io::Write;
use std::process::{Command, Stdio};

use crate::terminal::{ids_of, short_of, Live, Repo, AGENT, ONE_AT_A_TIME};

/// Wide enough for the dashboard, so the claims panel is drawn beside the
/// others and its rows can be read while another panel has the focus -- and
/// wide enough that the column of listings, two fifths of the window, holds the
/// suite's thirty-character holder whole beside both spans. A narrower row
/// drops the holder from the right, which the decision allows.
const WIDE: (u16, u16) = (200, 45);

/// The second identity, and a holder no other row of the corpus spells.
const OTHER: &str = "someone/else+claims-suite";

/// The second task's title, which no other entity shares.
const SECOND: &str = "Quartz lamps in the stairwell";

/// The suite's own claim on the seeded task and another identity's on a second
/// one, both taken by the binary.
fn two_holders() -> (Repo, String, String) {
    let repo = Repo::seeded();
    let mine = repo.only(&["--type", "task"]);
    repo.ank(&["claim", &mine]);
    repo.ank(&[
        "new",
        "task",
        "--title",
        SECOND,
        "--scope",
        "src/**",
        "--criteria",
        "Somebody else is on this one.",
        "--no-verify",
    ]);
    let theirs = ids_of(&repo.stdout(&["find", "--type", "task", "--json"]))
        .into_iter()
        .find(|id| id != &mine)
        .expect("the second task");
    let out = repo.tried(&["claim", &theirs], &[("ANK_AGENT", OTHER)]);
    assert!(
        out.status.success(),
        "the second identity's claim: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    repo.warm_find();
    (repo, mine, theirs)
}

/// Rewrites one field of a claim record and leaves every other byte of it.
fn forge(repo: &Repo, id: &str, field: &str, value: &str) {
    let name = format!("refs/ank/claims/{id}");
    let record = String::from_utf8_lossy(&repo.git(&["cat-file", "-p", &name]).stdout).to_string();
    let prefix = format!("{field}: ");
    assert!(
        record.lines().any(|l| l.starts_with(&prefix)),
        "the record carries no {field}:\n{record}"
    );
    let rewritten: String = record
        .lines()
        .map(|l| match l.starts_with(&prefix) {
            true => format!("{prefix}{value}\n"),
            false => format!("{l}\n"),
        })
        .collect();
    let mut child = Command::new("git")
        .args(["hash-object", "-w", "--stdin"])
        .current_dir(&repo.0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(rewritten.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let blob = String::from_utf8_lossy(&out.stdout).trim().to_string();
    repo.git(&["update-ref", &name, &blob]);
}

/// The lines of the claims panel's body: after its title and before the next
/// panel's.
fn claim_rows(frame: &str) -> Vec<String> {
    frame
        .lines()
        .skip_while(|l| !l.contains("1 CLAIMS"))
        .skip(1)
        .take_while(|l| !l.contains("2 ENTITIES"))
        .filter(|l| l.contains("TASK-"))
        .map(str::to_string)
        .collect()
}

/// The one claims row naming this task.
fn row_of(frame: &str, id: &str) -> String {
    let short = short_of(id);
    let rows: Vec<String> = claim_rows(frame)
        .into_iter()
        .filter(|l| l.contains(&short))
        .collect();
    assert_eq!(rows.len(), 1, "one claims row names {short}:\n{frame}");
    rows[0].clone()
}

/// What stands between the marker columns and the identifier on a row.
fn markers(row: &str, id: &str) -> String {
    let at = row.find(&short_of(id)).unwrap();
    row[..at].chars().skip(1).collect()
}

/// Opens the reader and waits until the claims have been read.
fn opened(repo: &Repo) -> Live {
    let live = Live::open(repo, WIDE.0, WIDE.1);
    live.until("the claims to have been read", |t| {
        claim_rows(t).len() == 2 && t.contains("ENTITIES all 3")
    });
    live
}

#[test]
fn a_claims_row_says_whose_since_when_and_how_long_it_has_left() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (repo, mine, theirs) = two_holders();
    // Taken years ago by the other identity, which is a claimed stamp nobody
    // can wait for and the one the age is read off.
    forge(&repo, &theirs, "claimed", "2020-01-01T00:00:00Z");
    let live = opened(&repo);
    let frame = live.frame();

    let own = row_of(&frame, &mine);
    assert!(own.contains(AGENT), "the holder is named:\n{own}");
    assert!(
        own.contains("s ago") || own.contains("m ago"),
        "a claim taken a moment ago says so:\n{own}"
    );
    assert!(
        own.contains("m left"),
        "a live lease says how long it has left:\n{own}"
    );
    assert!(!own.contains("lapsed"), "{own}");
    assert!(
        markers(&own, &mine).contains('*'),
        "the reader's own claim is marked:\n{own}"
    );

    let other = row_of(&frame, &theirs);
    assert!(other.contains(OTHER), "the holder is named:\n{other}");
    assert!(
        other.contains("d ago"),
        "a claim taken in 2020 is days old:\n{other}"
    );
    assert!(other.contains("m left"), "{other}");
    assert!(
        !markers(&other, &theirs).contains('*'),
        "another identity's claim is not marked as the reader's:\n{other}"
    );
    live.quit();
}

#[test]
fn a_lapsed_claim_says_lapsed_and_walking_the_claims_moves_no_ref() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (repo, mine, _) = two_holders();
    forge(&repo, &mine, "expires", "2020-01-01T00:00:00Z");
    let status = repo.stdout(&["status", "--json"]);
    assert!(
        status.contains("\"lapsed\":true"),
        "the fixture's claim is not lapsed by status's own account:\n{status}"
    );
    let refs = repo.refs();

    let mut live = opened(&repo);
    let own = row_of(&live.frame(), &mine);
    assert!(own.contains("lapsed"), "a lapsed claim says so:\n{own}");
    assert!(
        !own.contains("left"),
        "a lapsed claim has nothing left:\n{own}"
    );

    // Down the claims and back: the detail is drawn from what the listings
    // read, and a `show` would have renewed the lapsed claim's ref.
    live.send("1");
    for key in ["j", "j", "k", "k"] {
        live.send(key);
        live.frame();
    }
    assert_eq!(repo.refs(), refs, "walking the claims moved a ref");
    live.quit();
}

#[test]
fn the_search_narrows_the_focused_claims_as_it_is_typed() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let (repo, _, theirs) = two_holders();
    let mut live = opened(&repo);
    live.send("1");
    let frame = live.until("the claims to have the focus", |t| {
        t.lines()
            .any(|l| l.contains('\u{25b6}') && l.contains("1 CLAIMS"))
    });
    let keys = frame.lines().last().unwrap_or_default().to_string();
    assert!(
        keys.contains("[/ find]"),
        "the claims panel does not offer the search:\n{keys}"
    );

    live.send("/");
    live.until("the search to open", |t| t.contains("[Esc clear]"));
    let mut counts = Vec::new();
    let mut typed = String::new();
    for c in "someone".chars() {
        typed.push(c);
        live.send(&c.to_string());
        live.until(&format!("the needle to reach '{typed}'"), |t| {
            t.contains(&format!("/{typed}"))
        });
        counts.push(claim_rows(&live.frame()).len());
    }
    assert_eq!(
        counts.last().copied(),
        Some(1),
        "a needle only the other holder matches left {counts:?} claims rows"
    );
    let left = claim_rows(&live.frame());
    assert!(left[0].contains(&short_of(&theirs)), "{left:?}");

    live.send("\u{1b}");
    live.until("the claims to come back whole", |t| {
        claim_rows(t).len() == 2
    });
    live.quit();
}

#[test]
fn the_queue_offers_the_search_it_answers() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|held| held.into_inner());
    let repo = Repo::seeded();
    repo.warm_find();
    let mut live = Live::open(&repo, WIDE.0, WIDE.1);
    live.until("the session to open", |t| t.contains("4 QUEUE"));
    live.send("4");
    let frame = live.until("the queue to have the focus", |t| {
        t.lines()
            .any(|l| l.contains('\u{25b6}') && l.contains("4 QUEUE"))
    });
    let keys = frame.lines().last().unwrap_or_default().to_string();
    assert!(
        keys.contains("[/ find]"),
        "the queue does not offer the search it narrows by:\n{keys}"
    );
    live.quit();
}
