# Improving errors
Status: BLOCKED (error-chain)
Reason: More useful error messages and better error handling code.
Bloking something: NO


## Task
There are a couple of issues with the way errors are in replicante:

  * Nested error are too lengthty to log.
  * Top errors are often too generic.

Improvements are needed overall but are a large task so maybe delay to later?


### Reporting
The main problem with reporting is the full context is too long to spew on the screen and only
one massage is too little: top is too generic, leaf is too specific.

One possible solution:

  * Log already uses `error_message`, `error_cause` and layers.
  * Print to console the full trace (already done by `format_fail`).
  * Emit events to sentry for all other cases.


## Plan
TODO
