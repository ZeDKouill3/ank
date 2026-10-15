//! What a verb writes into the corpus (ADR-3c8d94a0bbe0).
//!
//! Here rather than beside the code that writes, because it is a field of
//! [`crate::CommandSpec`] for the reason [`crate::Renews`] is one: it is part of
//! what a verb *is*, and a surface describing the verbs describes it.

/// Where a verb leaves bytes, as far as the corpus is concerned.
///
/// ADR-e45e1a29fe91 enumerated the reading verbs and the writing verbs inside
/// its constraint, and the enumeration went stale once per new route: `archive`
/// and `migrate` wrote `.ank/` and sat on neither list. **A field is how the
/// compiler asks the question of every verb that is ever added**, and the
/// workspace suite confronts each verb declaring [`Writes::Nothing`] with what
/// it leaves on a scratch corpus, so the declaration cannot go stale silently
/// either.
///
/// The domain is the corpus and nothing else: a path under `.ank/`, and a ref
/// under `refs/ank/`. `skills --install` writes a directory elsewhere and
/// `update` replaces the binary, and neither touches a corpus. A verb that can
/// write on some path declares that path: `log` reads with an id alone and
/// writes with a message, and declares the write.
///
/// **A lease renewal is not counted here.** `show` on the task the caller holds
/// moves its claim ref, and [`crate::Renews`] already declares that per verb
/// (ADR-0bb7ea8991bc); counting it twice would put every verb that looks at the
/// held task on the writing side and leave the reading side meaning nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Writes {
    /// Neither: the verb reads the corpus, or works outside it.
    Nothing,
    /// Files under `.ank/` only.
    Files,
    /// Refs under `refs/ank/` only.
    Refs,
    /// Both, on one path or across its paths.
    Both,
}

impl Writes {
    /// The word `help --json` carries.
    pub fn as_str(self) -> &'static str {
        match self {
            Writes::Nothing => "nothing",
            Writes::Files => "files",
            Writes::Refs => "refs",
            Writes::Both => "both",
        }
    }
}
