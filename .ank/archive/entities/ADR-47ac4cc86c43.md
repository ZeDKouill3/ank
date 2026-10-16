---
id: ADR-47ac4cc86c43
type: adr
slug: ank-signers-sync-reads-a-principal-s-github-sign
title: ank signers sync reads a principal's GitHub signing keys through gh, and writes them as a diff to review
created: 2026-10-11T18:26:24Z
author: claude-code/opus-5.5
status: superseded
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/git.rs
  - .ank/config.yml
  - docs/signing.md
constraint: |
  A principal under signers: may carry github: <login>, and ank signers sync, an explicit network act and the only one reading GitHub, fetches that login's public ssh and gpg signing keys through gh api, adds those the corpus lacks with note: github, retires (valid_before today, never removes) those marked note: github that GitHub no longer serves, never touches a key added by hand, writes and commits nothing beyond config.yml so the keys bind only once a pull request lands them, and exits 9 naming gh auth login when gh is absent or unauthenticated; no HTTP client enters the binary, and check, accept and every other verb stay offline (ADR-5c6db7a161b8).
ratified: 1ccb272c7fc2
verified:
  - by: haksolot@omarchy
    at: 2026-10-11T19:04:21Z
schema: 4
version: 3
---

Builds on ADR-5c6db7a161b8, which moves signers into `config.yml` and makes
`ank signers` their writer.

## Context

Every maintainer of this repository already declares their signing keys on
GitHub, which shows a commit as Verified against them. Declaring the same key a
second time in the corpus is a copy by hand of a public key from one place to
another, and the copy is where the errors and the delay are.

The trust root stays the corpus: `check` must keep judging ratifications
offline, from what the repository holds (SPEC-15a56aeedcfd: no verb pays for
the network unless it is asked to). The binary links no HTTP client; every
network act it makes today goes through git.

## Decision

1. A principal under `signers:` may carry `github: <login>`.
2. **`ank signers sync [<principal>]` is an explicit network act.** It reads the
   login's public signing keys through `gh api users/<login>/ssh_signing_keys`
   and `gh api users/<login>/gpg_keys`, and writes the difference into
   `signers:`:
   - a key GitHub has and the corpus does not is added, with `note: github`;
   - a key the corpus marks `note: github` and GitHub no longer has is retired
     (`valid_before` today), never removed, so what it ratified stays verified;
   - a key added by hand is never touched.
3. **The result is a diff in review.** `sync` writes and commits nothing; the
   keys reach the default branch through a pull request like any other change
   to who may ratify, and only then can they ratify there.
4. `gh` absent or unauthenticated is exit 9 with the command to run (`gh auth
   login`). No HTTP client enters the binary.
5. `check`, `accept` and every other verb stay offline and never read GitHub.

## Consequences

- A new machine is: add the key on GitHub, `ank signers sync`, merge the pull
  request. No public key is copied by hand.
- A key deleted on GitHub stops ratifying at the next sync, without breaking
  what it signed.
- GitHub becomes a source of keys, not an authority: a key it serves still has
  to pass review before it binds anything.

## Rejected alternatives

- `check` resolving keys from GitHub live: removes the pull request, but makes
  the GitHub account the trust root and `check` a network verb.
- An embedded HTTP client: TLS and a heavy dependency in a tool that has none,
  for one explicit call.
- `curl`: not guaranteed on Windows, and it would still need a token for
  private rate limits that `gh` already holds.
- `https://github.com/<login>.keys`: those are authentication keys, not
  signing keys.
