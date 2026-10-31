//! **The launch-readiness suite** (`docs/Hyades_releases.md` §2). The release
//! workflow sets the major version to `floor(100 × passed / total)` over the
//! tests in this file, so each test is one criterion the game must meet to
//! launch, and major `100` is every criterion met.
//!
//! Every test here carries `#[ignore = "launch criterion"]`: a criterion fails
//! until the game meets it, and the CI gate must stay green meanwhile. The
//! release workflow runs them with `--include-ignored`. A criterion that has
//! passed at a release and fails later blocks the next release.
//!
//! The criteria are the author's to write; the suite starts empty, so major
//! starts at `000`. Each test obeys the 60-second rule (`AGENTS.md` §2).
