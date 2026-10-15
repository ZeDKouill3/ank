# Splitting a suite into verifiers

`ank done` runs every verifier the task declares, and a suite that takes ten
minutes makes every close cost ten minutes. This page is the recipe for cutting
that cost in your own repository, in any language: split the suite into one
verifier per package, tell each one which files it reads, and let `done` skip
what it has already seen pass. [Proof and verifiers](proof.md) is where
verifiers themselves are explained; this page assumes it.

Two mechanisms do the work, and they answer different questions:

- **`done` reuses by content.** A run is recorded under the verifier's
  definition and a digest of the files its `inputs` match. A later `done` whose
  key is the same, and whose recorded run passed, reuses it instead of starting
  the verifier again. Nothing about the branch, the commit or the order of
  events enters the key: the same bytes are the same answer.
- **`ank verify` selects by change, and proves nothing.** It runs the verifiers
  whose `inputs` match a file changed on the branch and records each run where
  `done` will find it. It needs no claim, writes no proof, no ref and no
  entity, and its selection is printed and never recorded. What it buys is
  that the `done` which follows has nothing left to run.

## A repository with two packages

Two packages, `billing/` and `search/`, each with a test script of its own,
`billing/test.sh` and `search/test.sh`. One verifier per package, each
declaring the files that can change its result, and both marked `default` so
every task carries them:

<!-- replay split ANK_AGENT=human:marie
$ mkdir -p billing search && echo 'exit 0' > billing/test.sh && echo 'exit 0' > search/test.sh
$ ank init && ank config default_branch main
-->

    $ ank config verifiers.billing-tests.run "sh billing/test.sh"
    verifiers.billing-tests.run (unset) -> sh billing/test.sh
    $ ank config verifiers.billing-tests.inputs "[billing/**]"
    verifiers.billing-tests.inputs (unset) -> ["billing/**"]
    $ ank config verifiers.billing-tests.default true
    verifiers.billing-tests.default false (default) -> true
    $ ank config verifiers.search-tests.run "sh search/test.sh"
    verifiers.search-tests.run (unset) -> sh search/test.sh
    $ ank config verifiers.search-tests.inputs "[search/**]"
    verifiers.search-tests.inputs (unset) -> ["search/**"]
    $ ank config verifiers.search-tests.default true
    verifiers.search-tests.default false (default) -> true

<!-- replay split part
$ cat .ank/config.yml
-->

    verifiers:
      billing-tests:
        run: sh billing/test.sh
        inputs: ["billing/**"]
        default: true
      search-tests:
        run: sh search/test.sh
        inputs: ["search/**"]
        default: true

**The inputs are disjoint, and that is the whole design.** A change under
`billing/` cannot alter what `search-tests` reads, so its last passing run
stays valid. A verifier with no `inputs` reads the whole tree: any change at
all invalidates it, and `ank verify` always selects it. In a real repository the
`inputs` of a package's tests are the package itself and whatever it depends on
inside the repository, shared code and lock files included; an input you leave
out is a change the reuse will not see.

## The first close runs everything

Nothing has been recorded yet, so the first task to close runs both:

<!-- replay split
$ git add -A && git commit -q -m "two packages"
$ git checkout -q -b task/ranking
-->

    $ ank new task --title "Rank exact matches first" --scope "search/**" \
        --criteria "An exact title match ranks above a partial one"
    created TASK-f89bb33affbf Rank exact matches first
    $ ank claim f89b
    claimed TASK-f89bb33affbf rank-exact-matches-first -> HEAD
    $ ank done
    running: billing-tests ... ok (0.0s)
    running: search-tests ... ok (0.0s)
    proof recorded: billing-tests@98f3ff3419f0 -> local/e3b0c44298fc@5f99e12  (scope/e2825598a6ca)
    proof recorded: search-tests@bd59a126e55a -> local/e3b0c44298fc@5f99e12  (scope/e2825598a6ca)
    TASK-f89bb33affbf -> done

Both runs are now recorded under `<git-common-dir>/ank/runs/`, which every
worktree of the repository shares.

## A change to one package

The next task changes `billing/` on a branch of its own:

<!-- replay split
$ git add -A && git commit -q -m "done ranking" && git checkout -q main && git merge -q task/ranking
$ git checkout -q -b task/rounding
-->

    $ ank new task --title "Round invoice totals half to even" --scope "billing/**" \
        --criteria "Totals round half to even"
    created TASK-5493c635b193 Round invoice totals half to even
    $ echo '# half to even' >> billing/test.sh
    $ ank verify
    running: billing-tests ... ok (0.0s)
    3 file(s) changed since ada2f49; ran 1 of 2 verifier(s), recorded for reuse
      billing-tests  ok
      search-tests  skipped, no input changed

`ank verify` read what changed since the merge-base with the default branch --
`billing/test.sh`, and the task and its log entry, not yet committed -- selected
the one verifier whose `inputs` that touches, and ran that one. It
proved nothing: the task is still `in_progress`, no proof was written, and
nothing about the selection was kept. What it kept is the run.

## The done that follows

<!-- replay split -->

    $ ank claim 5493
    claimed TASK-5493c635b193 round-invoice-totals-half-to-even -> HEAD
    $ ank done
    running: billing-tests ... reused (local, by human:marie)
    running: search-tests ... reused (local, by human:marie)
    proof recorded: billing-tests@98f3ff3419f0 -> local/e3b0c44298fc@ada2f49  (scope/9bd058bccab7)
    proof recorded: search-tests@bd59a126e55a -> local/e3b0c44298fc@5f99e12  (scope/9bd058bccab7)
    TASK-5493c635b193 -> done

Neither verifier started. `billing-tests` is the run `ank verify` just
recorded, on the bytes `billing/` holds now. `search-tests` is the run the
previous close recorded: `search/` has not changed since, so the key is the
same and the answer is the same. The commits in the two entries say so: `ada2f49` is
where `ank verify` ran, `5f99e12` where the first close did. Each entry keeps
the reference and the route of the run it came from and adds `reused: local`
and `by:`, so the record says which closes ran a verifier and which took an
earlier answer.

## What is never reused

A failure is never reused, nor a run from another machine, nor an attestation,
nor a run older than `proof.reuse_max_age` (default `24h`). `ank done --fresh`
runs every verifier regardless. And `ank verify` is never a substitute for
`done`: a task closes only when `done` has recorded its proof, whether the runs
behind that proof were started by `done` itself or found waiting.
