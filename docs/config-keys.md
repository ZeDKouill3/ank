<!-- Generated from crates/ank-core/src/config.rs; do not edit.
     Regenerate: cargo run -q -p ank-core --bin config-keys > docs/config-keys.md -->

# config.yml keys

Every key `.ank/config.yml` may carry, with the type of its value and what an absent key means. A key not listed here is refused, and so is a file whose `schema` is newer than this build reads.

The file declares `schema: 1`, the only version this build reads. A duration is `<n><unit>`, the unit one of `s`, `m`, `h` or `d`. `<name>` stands for a key the file chooses; `ank config <key>` reads and writes the scalar keys, and `roles` and `identities` are edited by hand.

| Key | Type | Default | Notes |
|---|---|---|---|
| `schema` | integer | required | the version of this file's format |
| `context_budget` | integer | `8000` | what `context` hands a reader, in characters |
| `claim_ttl_max` | duration | `2h` | the longest lease a claim is granted, whatever `--ttl` asks |
| `claim_ttl_default` | duration | `30m` | the lease `claim` grants without `--ttl`, capped by `claim_ttl_max` |
| `default_branch` | string | none | the branch carrying the reference state; absent, `refs/remotes/origin/HEAD` names it |
| `peers.<name>` | path | none | a peer corpus a scope entry reaches by name: a path to a checkout on disk, relative to this root or absolute, and never a URL; a reader whose layout differs overrides it in their own corpora.yml with `ank config --user peers.<identity>.<name> <path>` |
| `verifiers.<name>.run` | command | required | what `done` runs through `sh`; required in a declared verifier |
| `verifiers.<name>.timeout` | duration | `10m` | how long `done` lets the command run |
| `verifiers.<name>.default` | boolean | `false` | `true` writes the verifier into every task `ank new task` creates |
| `verifiers.<name>.inputs` | list of globs | none | the files whose content can change the result, relative to the root; absent, the whole tree; written as one flow sequence, never empty |
| `verifiers.<name>.env.image` | string | none | the OCI image the verifier is meant to run in, pinned as `<name>@sha256:<digest>`; a tag is refused; ank never runs it |
| `verifiers.<name>.env.platform` | string | none | `os/arch`, required with `env.image` |
| `verifiers.<name>.env.network` | `none` or `open` | none | absent, `none` under an image and the host's network without one |
| `verifiers.<name>.env.vars.<var>` | string | none | a variable with a fixed value; nothing is passed through from the caller |
| `verifier_jobs` | integer | none | how many verifiers one `done` runs at once, at least 1; absent, the machine's available parallelism |
| `roles.<name>.can` | list of strings | `[]` | what the role may do, declared |
| `roles.<name>.cannot` | list of strings | `[]` | what the role may not do, declared |
| `identities.<identity>` | string | none | the role of an identity; one absent from the table is an `agent` |
| `weight.hot_files` | integer | `3000` | `check` signals a hot corpus holding more entity files |
| `weight.plane_bytes` | integer | `4000000` | `check` signals claim and proof records weighing more bytes |
