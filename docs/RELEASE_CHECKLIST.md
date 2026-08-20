# Foundation v0.1.3 release checklist

> Status: maintainer checklist, not a release announcement
>
> Scope: `refetch-project/core-rust`
>
> This checklist prepares a repository tag/release. It does not authorize a push, tag, GitHub release, or crates.io publication.

## 1. Declared compatibility

| Axis | Locked value |
| --- | --- |
| JSON contract | `v0.1` |
| Foundation revision | `v0.1.3` |
| Concept commit | `823c5303246b467fe9425141c1dcbca92537db28` |
| Rust workspace version | `0.1.0` |
| Rust toolchain | `1.97.1` |
| Algorithm ID | `refetch.rank.baseline.v0.1` |
| Invalid fixture count | `36` |

Rust workspace `0.1.0` is the reference implementation of the locked Foundation v0.1.3 snapshot above. This statement does not imply that the crates have been published or that later Concept commits are compatible.

## 2. Repository baseline

Run from a fresh checkout or detached worktree of the exact release candidate:

```bash
git status --short --branch
git remote -v
git rev-parse HEAD
git rev-parse origin/main
git merge-base HEAD origin/main
cargo --version
rustc --version
```

Release evidence requires:

- [ ] `origin` is `https://github.com/refetch-project/core-rust.git`.
- [ ] The checkout is clean and contains no ignored credentials or generated samples.
- [ ] The release candidate commit is an explained descendant of `origin/main`.
- [ ] `cargo --version` and `rustc --version` report `1.97.1`.
- [ ] The candidate contains no platform-specific Core branch, network runtime, model call, FFI, or product sample.

## 3. Concept provenance

Use a separate clean Concept checkout at the exact lock commit. Do not move or modify the maintainer's primary Concept worktree merely to satisfy this checklist.

```bash
git -C /path/to/concept status --short --branch
git -C /path/to/concept rev-parse HEAD
python3 /path/to/concept/scripts/validate-fixtures.py
python3 scripts/verify-spec-snapshot.py
```

Expected evidence:

- [ ] Concept HEAD is exactly `823c5303246b467fe9425141c1dcbca92537db28` and the checkout is clean.
- [ ] Concept validator reports `3` valid and `36` invalid fixtures.
- [ ] The snapshot contains exactly the same `docs`, `fixtures`, `rfcs`, and `schemas` files as the locked commit.
- [ ] Snapshot manifest verification succeeds with no missing, mismatched, or extra files.

## 4. Core acceptance chain

Run the complete chain after the final content change:

```bash
python3 scripts/verify-spec-snapshot.py
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo build --workspace --release --locked
cargo test -p refetch-core --test conformance --locked invalid_fixtures_fail -- --exact --nocapture
git diff --check
```

Expected evidence:

- [ ] Workspace tests report `36` passed and `0` failed across all test binaries.
- [ ] Conformance output reports `36` invalid fixtures discovered and `36` executed.
- [ ] Three valid fixtures reproduce their complete expected `FeedSlate` values.
- [ ] `cargo fmt`, locked clippy with `-D warnings`, release build, and `git diff --check` succeed.
- [ ] `crates/refetch-core/src/lib.rs` has no unexplained ranking or selection change relative to the reviewed baseline.

## 5. Packaging boundary

Foundation v0.1.3 can be tagged as a repository reference implementation before crates.io publication is ready. Treat these as separate gates:

- [x] This Foundation candidate is repository-only; crates.io publication is outside the current release scope.
- [ ] If publishing crates, add and verify all required package metadata and versioned path dependencies.
- [ ] Run `cargo package` and `cargo publish --dry-run` for each intended crate in dependency order.
- [ ] Inspect packaged file lists for snapshots, credentials, build output, and unrelated product data.

Current packaging audit:

- `refetch-contract` packages successfully but warns that description, documentation, homepage, and repository metadata are absent;
- `refetch-core` and `refetch-cli` do not package because their local `refetch-*` dependencies have no crates.io version requirement;
- these are crates.io publication blockers, not failures of the offline repository reference implementation.

Do not describe a repository tag as a published Rust package.

## 6. Claims and remaining boundaries

Before release, confirm the notes state:

- [ ] Core accepts complete normalized `RankRequest` input and returns deterministic `FeedSlate` output.
- [ ] JSON parsing failures and `RankError` remain separate boundaries.
- [ ] The CLI is a file-based offline experiment boundary, not a stable machine protocol.
- [ ] No live Adapter, Analyzer, Bilibili/PiliNara integration, Flutter FFI, WASM, database, cloud service, or AI runtime is included.
- [ ] Local and CI conformance do not prove product value, privacy safety of Host exports, or real-device behavior.
- [ ] Known limitations and deferred decisions are listed explicitly.

## 7. Authorized release actions

Only after the evidence above is attached to the exact candidate commit and the maintainer separately authorizes remote actions:

1. push the reviewed branch;
2. verify required CI on the pushed commit;
3. create the approved tag;
4. create release notes that match the verified boundaries;
5. verify the remote tag and release artifact;
6. publish crates only if the independent packaging gate is complete.

Record the final commit SHA, tag, CI run, fixture counts, Rust test counts, and unresolved issues. A local commit, successful test run, pushed branch, tag, GitHub release, and crates.io publication are distinct states.
