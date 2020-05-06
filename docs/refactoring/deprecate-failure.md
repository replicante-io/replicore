# Deprecate failure crate
Status: BLOCKED (see below)
Reason: https://internals.rust-lang.org/t/failure-crate-maintenance/12087
Blocking something: NO


## Blocked by

  * Need for more features (same as async-rewrite.md).
  * Failure is pervasive and has very few dependencies it blocks from updating.


## Task

  * The `failure` crate is deprecated and I need to find a replacement.
  * First look at suggested alternatives and spec out needs and options (thiserror, anyhow, eyre, snafu).
  * Prototype replacement (focus: backtraces support, user display, sentry report, actix-web integration).
  * Figure out replacement path.
