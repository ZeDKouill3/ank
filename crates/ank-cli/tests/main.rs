//! The one integration test target of `ank-cli`: every file beside this one is
//! a module of it, so the dependency tree is linked once rather than once per
//! file (TASK-22af5b0918a4). `Cargo.toml` sets `autotests = false` and names
//! this file, so a new suite is a new `mod` line here and never a new binary.
//!
//! The helpers are declared here, once, and each suite reaches them as
//! `crate::scratch`, `crate::fixture` and `crate::index_file`.

mod fixture;
mod index_file;
mod scratch;

mod accept_pr;
mod adopt;
mod adr_amends;
mod amend_amends;
mod amends_context;
mod amends_cost;
mod amends_json;
mod archive_checkout;
mod archive_digest;
mod attest_ids;
mod cli;
mod context_since;
mod corpora_schema;
mod doc_replay;
mod guide;
mod help_new;
mod index_per_schema;
mod init_at;
mod init_origin;
mod init_refspec;
mod init_wal_ignore;
mod log_rewritten;
mod mcp;
mod peer_blocked_by;
mod peer_claimable;
mod peer_ids;
mod peer_override;
mod peer_scope;
mod peer_scope_reverse;
mod peer_url_refusal;
mod remote_without_claim;
mod renewal_noop;
mod schema;
mod skill;
mod spec_page;
mod status;
mod tui;
mod update;
mod verifier_env;
mod warm_readers;
mod watch;
