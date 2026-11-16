## TL;DR

Use this index to find CI checks and artifact publication workflows.

# CI/CD workflows

Every GitHub Actions workflow in the repository: the pull-request checks, the SDK build and
publish chain, the local end-to-end suite, and the box images.

Shared step bundles live one directory over, in [`.github/actions/`](../actions) — GitHub does not
support subdirectories under `.github/workflows/`, so composite actions are where reuse goes.

## How they fit together

```text
PULL REQUEST / PUSH                     lint · test · codeql
                                        e2e-local · build-box-images

BUILD + CACHE (weekly)                  build-runtime

RELEASE CHAIN (workflow_run)            build-c ──▶ build-go

SDK TEST BUILDS (manual dispatch)       build-go ──▶ build-c ──▶ Go SDK tests

RELEASE (release event)                 build-runtime · build-c · build-node · build-wheels
                                        apps/box-images/v* tag ──▶ release-box-images

CONFIG                                  ci-config action ◀── lint, test, config workflow
                                        config workflow ◀── build workflows
```

## Workflows

**Callable** marks a workflow another one can invoke with `uses:`. `config.yml` is the one that is *exclusively* callable; workflows with `workflow_dispatch` can also run on their
own.

| Workflow | Triggers | Callable | Purpose |
| --- | --- | --- | --- |
| `config.yml` | `workflow_call` | call-only | Loads `.github/ci-config.json` before build matrices expand |
| `lint.yml` | push, PR, merge_group | — | Format and lint per language, plus the installer smoke test. `Lint (conclusion)` is the required check |
| `test.yml` | push, PR, merge_group, weekly, dispatch | — | Lightweight draft checks; SDK tests and combined Rust/CLI coverage on ready PRs. Compact routine matrices and full weekly/manual matrices. Codecov requires 90% patch coverage |
| `codeql.yml` | push, PR, dispatch, weekly | — | CodeQL advanced setup, so fork PRs are scanned |
| `author-review.yml` | PR (target), issue_comment, merge_group | — | Converts unacknowledged PRs to draft, posts author instructions, and publishes `Author reviewed the PR` on the current head. Merge queues carry forward the required PR admission check |
| `build-runtime.yml` | weekly, release, dispatch | — | Builds runtime/CLI artifacts and populates sccache together; publishes crates on release |
| `build-c.yml` | release, dispatch, `workflow_call` | yes | C SDK archives |
| `build-go.yml` | `workflow_run`, dispatch | — | Pins release tests and the Go tag to the C build commit. Compares public archives with that run before testing. Manual tests build their own C archive. |
| `build-node.yml` | release, dispatch | — | Node.js SDK, napi-rs addon and platform packages |
| `build-wheels.yml` | release, dispatch | — | Builds Python wheels and verifies their native extension in cibuildwheel before publishing |
| `e2e-local.yml` | push, `pull_request_target`, dispatch | — | VM-based tests on a self-hosted GCP VM runner. Needs `/dev/kvm`; PRs need the `e2e-local` label |
| `build-box-images.yml` | PR, push, dispatch | — | Builds changed flavors for both arches; shared inputs and manual runs build every flavor |
| `release-box-images.yml` | `apps/box-images/v*` tag, dispatch | — | Publishes the curated box image flavors to GHCR |

Longer treatments live with their subject rather than here: [E2E local
runbook](../../docs/contributing/development/e2e-local.md).

## Routine checks

`lint` and `test` always report their required conclusion on PRs and merge groups. A newer PR
commit cancels its superseded lint and test runs. Release jobs keep their existing sequencing.

Joining a [native GitHub stack](https://docs.github.com/en/pull-requests/tutorials/roll-out-stacked-prs#5-update-your-programmatic-tooling)
emits `pull_request.stacked`, which starts Test with the existing suite and draft filters.
GitHub evaluates native stack PRs against the stack's trunk, so `branches: [main]`
covers upper layers when the trunk is `main`; every layer needs `Test (conclusion)`.
Adding the trigger does not retroactively start missing runs for PRs already in a stack.

Draft PRs defer the Rust, Python, Node and Go SDK coverage matrices, including Go
vet and golangci-lint, which reuse the Go job's native build. The Lint and Format
workflow (including Go formatting), CodeQL, managed Code Quality, API tests and
setup checks retain their existing selection. Workflow and infrastructure changes
run `make coverage:infra` in a separate, path-filtered job whose failures block
`Test (conclusion)`. It preserves tooling typechecks and all infrastructure suites, collects
TypeScript source coverage with c8, and uploads `target/coverage/infra/lcov.info` using OIDC
under the `infra` flag. These changes cannot select the empty-upload path.
The `nix` job builds the CLI in the flake dev shell, with the runner's
environment and Rust homes dropped, and boots `alpine:latest`. It runs when the
flake or an input of `make cli` changes (Make recipes, build scripts,
`src/deps/`, `build.rs`, Cargo manifests, the toolchain), or on manual
dispatch; not weekly, because `flake.lock` pins its inputs. It is not yet in
`Test (conclusion)`, so its failures show on the PR without blocking merge; it
becomes required once it has proven stable. It compiles through sccache but
does not cache `/nix/store`. Ubuntu still has `/usr/lib`, so it misses
regressions that break only on NixOS.
Marking a PR **Ready for review** triggers the SDK matrix and both Go analyzers
without requiring another commit. When a user returns it to draft, the new run
cancels the prior Test run and runs the lightweight selection. The author-review bot uses `GITHUB_TOKEN`;
[GitHub suppresses that token's draft-conversion event](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#triggering-a-workflow-from-a-workflow),
so automatic conversion cannot cancel an already-running matrix. Main pushes,
merge groups, weekly runs and manual dispatches retain SDK testing.
Deferred coverage may remain pending on a draft; it is never replaced by
an empty upload when coverage-bearing files changed.

Python runs all four supported versions on Linux x64 and the latest on macOS and Linux ARM
(six jobs); Node runs all three versions on Linux x64 and the latest on the other platforms
(five jobs). Weekly and manually dispatched tests exercise every platform/version combination
and bypass change filters. Combined Rust and CLI coverage runs on all three platforms, including
non-VM integration tests and Linux guest tests. Codecov requires 90% coverage of changed lines
and reports total coverage.

The Linux Rust jobs enable unprivileged user namespaces on their disposable runners
so SSH guest tests can create private user/mount namespaces. This temporarily disables
Ubuntu's AppArmor user-namespace restriction, as the Nix job does for its jailer.
Namespace setup failures remain test errors.

`make coverage:bindings:external` runs `make test:unit:scripts` before collecting
native SDK profiles. The infra job also runs `make test:unit:scripts` whenever
`tests/scripts/` or a script it drives (`scripts/ci/`, `scripts/test/coverage-bindings.sh`)
changes. A relative `CARGO_TARGET_DIR` is resolved from the repository
root once, so Python and Node builds share the report's object directory.

Linux x64 also runs the native KVM hardware tests and adds their profiles before
the Rust coverage upload. Missing `/dev/kvm` access fails this qualification.
When a change matches the `vmm` filter (the VMM and hypervisor crates, the kernel
build and its Make recipes), and on the weekly and manual runs, the job also runs
the VMM's real-kernel boot tests with `make coverage:vmm:boot`. It restores the
pinned `vmlinux` from the Actions cache, keyed on the kernel pins and build
script, or builds it with `make vmm:boot:host` on a miss.
The boot coverage target also builds the static production guest mount test
init with `make vmm:guest-mount-init` and exports `VMM_GUEST_MOUNT_INIT` to
the tests. This keeps the guest mount test in the boot coverage suite rather
than skipping it. The selected Linux x64 job installs the musl Rust target,
musl compiler and gperf, and uses the musl compiler for Rust linking and C
dependencies. The boot step's five-minute budget includes the release build
of this init; its guest execution remains bounded by the test watchdog.

`API Tests` also runs the shared CI Listener and standalone host suites, using
the job's disposable PostgreSQL service through the same `DB_HOST`, `DB_PORT`,
`DB_USERNAME`, `DB_PASSWORD` and `DB_DATABASE` settings as other API tests.
Changes to either CI project select this job on drafts and ready PRs. Their
LCOV reports are uploaded with the API report under the `api` flag; local checks
use `make coverage:ci-listener` and `make coverage:github-ci-runner`. Leaving the
database URL unset skips the PostgreSQL contracts locally, so that run cannot
certify hosted persistence or rollback behavior.

When none of the coverage suites is selected on a PR or merge group, the
`Codecov (no coverage changes)` job runs a validated `empty-upload`. Codecov
checks the changed files before publishing a passing or failing status; the
workflow never forces a pass. Failed file detection cannot select this path,
and upload errors fail `Test (conclusion)`. Source changes still need reports
and 90% patch coverage; missing reports fail the patch status.

Plugin resources, `scripts/plugins`, `tests/plugins`, and their Make/CI inputs select
the lightweight `Plugin package coverage` job, including on drafts. It measures
package tests and real check/dist commands through `make plugin:acrylmicrovm:coverage`,
saves the Cobertura XML as the `plugin-coverage` artifact, and uploads it with OIDC
under the `plugins` carryforward flag. Selected plugin source cannot take the
empty-upload path; package or upload failures block `Test (conclusion)`.

The package validator requires onboarding to name a distributed, frontmatter-validated
`skills/<name>/SKILL.md` inventory. Excluded caches and shipped reference files cannot
serve as onboarding skills, even when both manifests name the same existing file or alias.
Each onboarding regression restores original manifest bytes and removes its invalid resource
before the next case, so leftover invalid state cannot mask a later failure.

After verifying normal uploads and the skipped-coverage path on PRs and merge
groups, require `codecov/patch` from the Codecov app in the main ruleset alongside
`Test (conclusion)`. Deploy the workflow before enabling that requirement so
existing documentation-only PRs do not wait for a status they cannot publish.

Go vet and golangci-lint run after Linux x64 Go coverage, reusing its native SDK build.
Go module-download caches hash the repository's nested `go.mod` and `go.sum` files;
there is no root `go.sum`, and the SDK needs its `go.mod` included even without a
checksum file. The Go build cache is not restored because it cannot detect changes
to externally linked CGO libraries such as `libacrylmicrovm.a`.
Formatting-only jobs disable module caching. Go test jobs initialize
submodules before Rust cache setup and retain build state for both the root Cargo
workspace and vendored libkrun. They retain workspace compilation outputs, while
still invoking `make coverage:go` to rebuild changed code and current Git metadata.
Finished native libraries are not reused without Cargo's freshness checks.
The Go lint job retains formatting checks. SDK and API test filters exclude Markdown and select
the Make recipes they execute; the changes job still parses every Make include with `make -n help`.
Client drift and VM E2E triggers also exclude Markdown-only changes.

Client drift checks watch API code, shared libraries, generators and workspace configuration.
Guest artifact qualification runs only weekly or when manually dispatched, retaining all five
platform/profile combinations. PR, push, and merge-queue runs skip that job; use
`make test:guest-artifacts` to check guest build changes locally before merging. Infrastructure
tests run in hosted CI for infrastructure, workflow and relevant Make changes, including drafts,
and remain available through `make test:apps:infra` and the local pre-push check;
`make test:apps:infra-config` explicitly installs and type-checks the SST configuration.

## Author review gate rollout

Require the commit status `Author reviewed the PR` from GitHub Actions on the target
branch after this workflow is deployed. The handler job `Update author review status`
only reports whether event processing succeeded; it is not the acknowledgment.

Post `/recheck-author-review` as a PR comment to initialize existing PRs or retry a failed
handler. Any new non-bot PR comment rechecks live state without acknowledging the diff.
Comment events run the default-branch workflow; rechecks cannot select a modified branch
workflow. Bot instruction edits and deletions are reconciled too.
The bot comment includes the exact command the author must post. No fork branch writes,
extra GitHub App, or personal token are needed. The workflow runs only the immutable
upstream revision in `AGENT_TOOLING_REV`; update that pin through a reviewed PR.

Draft conversion requires `contents: write` as well as `pull-requests: write`;
`statuses: write` publishes the acknowledgment. The contents permission authorizes
the GraphQL mutation; the workflow does not push commits or write to fork branches.

Merge queues must require the same PR status before admission. Queue commits carry
that result forward; authors acknowledge their own PR head, not the temporary merge.
Unacknowledged PRs are converted to draft. After the author acknowledgment passes,
click **Ready for review** when reviews are wanted; acknowledgment preserves the draft
state. A new commit or editing/deleting the only acknowledgment returns the PR to draft.

## Composite actions

In [`.github/actions/`](../actions). Each replaces a step bundle that was previously copied into
every consumer.

| Action | Sites | Used by |
| --- | --- | --- |
| `ci-config` | 3 | config, lint, test |
| `setup-rust` | 10 | build-c, build-node, build-runtime ×2, build-wheels, lint ×2, test ×3 |
| `sccache` | 7 | build-c, build-node, build-runtime, build-wheels, lint, test ×2 |
| `build-guest` | 4 | build-c, build-node, build-runtime, build-wheels |
| `upload-to-release` | 5 | build-c, build-node, build-runtime, build-wheels |
| `run-in-manylinux` | 3 | build-c, build-node, build-runtime |
| `setup-go` | 4 | build-go, lint, test |
| `setup-python` | 5 | lint ×2, test ×3 |
| `setup-buildx` | 2 | build-box-images, release-box-images |

Two ordering rules, stated in each action's own header: `sccache` runs after `setup-rust`, and
`build-guest` and `run-in-manylinux` run after both. They are separate actions rather than one
because sccache is job-scoped — `run-in-manylinux` mounts the sccache binary and reads the
variables `sccache` exported, long after `build-guest` is done with them.

## sccache

Jobs invoking [the sccache action](../actions/sccache/action.yml) use sccache 0.17.0 with a
1 GiB local compiler cache, restored and saved as one GitHub Actions archive per job. This
replaces per-compilation remote writes, which were failing across native builds. Restore keys
prefer the same workflow/job and lockfiles, then fall back to the same runner OS/architecture.
Each successful run saves a new snapshot; compiler content hashes still decide whether entries
are reusable. The lockfile hash is captured before compilation, so saving the archive does not
scan private lockfiles created by root containers. A cold cache still requires a full build.

- The action sets `SCCACHE_GHA_ENABLED=false`, `RUSTC_WRAPPER=sccache` and
  `CARGO_INCREMENTAL=0`. Direct preprocessing mode is disabled when reusing archives.
- Host and Linux containers share `$RUNNER_TEMP/sccache` (mounted at `/cache/sccache` in
  manylinux). It stays outside the checkout so cibuildwheel does not copy a large cache with
  the sources. Only one server may use it at a time:
  the host stops before the container starts, and the container flushes its cache before exiting.
  Root-owned container entries return to the runner user before the archive is saved.
- `SCCACHE_BASEDIRS` normalizes source paths: the workspace on the host, `/work` in
  manylinux, and `/project` in cibuildwheel. Different compilers or build flags still produce
  different cache keys.
- cibuildwheel copies the host action's verified binary and accesses its cache through the
  `/host` mount. Standalone builds use an existing binary or PyPI's published 0.16.0 package.
  Its absolute wrapper survives Python-specific PATH changes. A local wheel
  build without a host directory uses a temporary cache inside the container.
- Host setup exports the wrapper only after the server starts. Ordinary jobs warn and compile
  uncached on setup failure; scheduled runtime builds require successful setup. Containers drop
  a missing wrapper (manylinux) or install a pass-through shim (cibuildwheel). A server failure
  inside a container can still fail that build. Host diagnostics go to
  `$RUNNER_TEMP/sccache-error.log`; post steps print statistics before archiving.
- Rust coverage and guest qualification use the target-directory cache provided by
  `setup-rust-toolchain`, rather than this sccache action.
- Linux runtime, C, Node and wheel distribution jobs disable that separate Cargo cache:
  guest staging discards the host toolchain and target directory before container builds.
  Their shared compiler cache remains enabled. macOS and other jobs retain Cargo caching.

Runtime, C, Node and wheel release builds can reuse compiler snapshots for their platform.
They still assemble their own SDK artifacts and perform cold builds when no compatible entry
exists. Runtime distribution builds run weekly, manually and on releases; routine tests and
Clippy continue checking platform-specific code.

## Box image cache

Both box-image workflows export Buildx runtime cache credentials through `setup-buildx`.
`apps/box-images/build.sh` imports and exports a separate GitHub Actions layer cache for each
image flavor. Export failures are tolerated and bounded to three minutes. Local builds without
GitHub credentials do not use the remote cache.

## CodeQL

`codeql.yml` uses CodeQL **advanced** setup rather than default setup, because default setup does
not analyze pull requests from forks — which makes the `code_scanning` ruleset rule ("Require code
scanning results") permanently block fork PRs. Advanced setup runs on `pull_request`, so fork PRs
in this public repo are scanned and the gate is satisfiable without an admin bypass.

Markdown-only pushes and PRs skip CodeQL; any code change retains the full language matrix.
Weekly and manual scans remain unfiltered. The required security ruleset stays in place;
GitHub's separate managed Code Quality analysis is unchanged.

The `analyze` job is a matrix over `actions`, `c-cpp`, `go`, `javascript-typescript`, `python` and
`rust`. All use `build-mode: none` (source only, no compile) except `go`, whose extractor has to
observe a real build and therefore uses `autobuild`.

## Manual SDK builds

Dispatching `build-go.yml` builds the selected workflow ref's C SDK on all supported platforms, then tests
each Go SDK against its matching artifact. The manual path does not require a published release for the
current `Cargo.toml` version. The release chain still downloads the published C archives. A C build failure
stops its consumers; manual runs publish no releases or tags.

Go module tagging runs only after successful release-origin C SDK builds and a successful Go test matrix.
The manual `build-c` job is intentionally skipped on that path, so the publication job uses
`!cancelled()` and explicit prerequisite-result checks to override the implicit `success()`
check inherited through the dependency chain. Failed or skipped prerequisites and workflow
cancellation still prevent publication.

## Do not rename these

A workflow chains off another's **display name**, not its filename. Changing the `name:` below
silently stops the chain — no error, the downstream workflow simply never fires.

| `name:` | Depended on by |
| --- | --- |
| `Build C SDK` | `build-go.yml` |

Go release tests require the canonical C workflow and read the platform matrix and toolchain
at that run's commit. A missing or different C run archive stops testing. Before creating
the Go module tag, the workflow checks that the remote product tag still points to the tested commit.

## Publishing secrets

Repository secrets, in Settings → Secrets and variables → Actions:

- `CARGO_REGISTRY_TOKEN` — crates.io, for the Rust crates
- `PYPI_API_TOKEN` — PyPI, for the wheels
- `NPM_TOKEN` — npm, for the Node packages
- `GH_APP_PRIVATE_KEY` — GitHub App key that registers the self-hosted E2E runner
