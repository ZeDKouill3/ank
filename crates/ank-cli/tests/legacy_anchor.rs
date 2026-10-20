//! The `ratify <id>` commit an `accept` made before ADR-5e47007a75be, made
//! again by hand over what `accept` writes now.
//!
//! `accept` commits nothing since that decision: the ratification is a tag
//! object at `refs/ank/ratify/<id>`. The readers that judge a ratification
//! (`check`, `context`, `scope`) still find it by the commit walk until
//! TASK-6e3f8f2ff6c9 teaches them the ref, and the walk stays afterwards as the
//! fallback for every anchor made before the decision. So a suite that tests
//! one of those readers ratifies with `accept` and then makes the commit the
//! old `accept` would have made: the subject and the lines are the tag's
//! message, which is the commit message unchanged, and the files are what
//! `accept` wrote.

/// Commits `.ank/` under the message of the tag `accept` just wrote for `id`,
/// signed with `-S` when `sign` holds. `git` runs one git command in the
/// suite's own environment and returns its standard output.
pub fn commit(git: &dyn Fn(&[&str]) -> String, id: &str, sign: bool) {
    let tag = git(&["cat-file", "tag", &format!("refs/ank/ratify/{id}")]);
    let message = tag
        .split_once("\n\n")
        .map(|(_, m)| m)
        .expect("a tag object has a message")
        .split("-----BEGIN")
        .next()
        .unwrap()
        .to_string();
    git(&["add", "-A", ".ank"]);
    let mut args = vec!["commit", "-q", "-m", message.as_str()];
    if sign {
        args.insert(1, "-S");
    }
    git(&args);
}
