---
id: LOG-1f7ec35c51f0
type: log
title: "Measured, full ank-cli suite after the repair: 5 red, all fixtures that model an unfetched"
created: 2026-10-12T12:44:48Z
author: claude-code/opus-5.5
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/status.rs
  - docs/claims.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/init_refspec.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/watch.rs
about: TASK-b55c33df5aa3
seq: 5
schema: 4
version: 1
---

 coordination plane as a clone with no refs/ank refspec (cli.rs: status_remote_names_the_claims_origin_holds..., status_names_the_refs_this_checkout_holds..., a_claim_a_watcher_mirrored..., neither_check_nor_status_fetches...; watch.rs: a_claim_taken_in_one_clone_is_reported_by_status_in_the_other). Such a clone now repairs itself on its first status or check, so the steady state those tests are about is a clone that has the refspec and has not fetched since. Fix: Repo::cloned installs the refspec in both repositories, as init would once origin exists, and watch.rs's second clone gets it too. Also observed: the repair fetch is forced (+refs/ank/*), so in status_names_the_refs... it overwrote a local claim ref with origin's moved one, exactly as a plain git fetch with init's refspec does; a fresh clone has nothing local to lose. Scope widened to crates/ank-cli/tests/watch.rs.
