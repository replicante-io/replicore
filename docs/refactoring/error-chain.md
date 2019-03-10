# Error Chain deprecation
Status: **ONGOING**
Reason: Access to new featues, keep up with community
Bloking something: YES (sentry integration, better error reporting)


## Task
Remove all uses of the `error-chain` crate in favour of the `failure` crate.

Requirements:

  * Support dependencies that use `error-chain`.
  * Use the "Error and ErrorKind pair" pattern.
  * "Leaf" libraries should use the "Custom Fail type" if their errors can be easly enumerated.
  * Try to avoid overly generic error messages.


## Crates
Migration and changes to error handling and returend errors are managed one crate at a time.
This keeps changes contained and better isolated.

Migration from `error-chain` to `failure` is easier top-down because `failure` can and should
deal with `error-chain` but the other way around is more complex.
The dependency graph was generated with `cargo-deps`, filtering for the crates below only.
Reversing the dependency order gives the migration order.

### Core

  1. [x] replicante_agent_discovery
  2. [x] replicante_data_fetcher
  3. [x] replicante_agent_client (+ rearrange metrics code)
  4. [ ] replicante_streams_events
  5. [ ] replicante_data_store

### Agents

  1. [ ] replicante_agent_mongodb
  2. [ ] replicante_agent_zookeeper
  3. [ ] replicante_agent_kafka
  4. [ ] replicante_agent

### Common

  1. [ ] replicante_util_tracing

### Finally

  1. [ ] Remove any unnecessary `SyncFailure`s
  2. [ ] Always use `failure_info` to log error information


## Plan
These steps are repeased for each create that needs to be migrated:

  1. Remove `error-chain` from Cargo.toml file and `extern crate`.
  2. Add `failure` to Cargo.toml and `extern crate`.
  3. Replace the `error(s?)` module with an `error` module that uses the new patter.
  4. Run tests for that crate and be dismayed.
  5. Fix the crate to use `failure` and the new error.
  6. Run all tests tests and be dismayed again.
  7. Fix all issues to complete the crate migration.


### Wrapping error-chain errors
Should integrate natively, with `#[cause]` and other tools, but if not there are some options.
REVIEW THIS IF/WHEN NEEDED.

  * Create a specialised wrapper `Fail` implementation (forward cause and the like).
  * Convert an `error-chain` into a chain of `Base(String) | Chain(String, Box<Self>)`.
  * Convert the full chain into a string.
  * Convert the full chain into a string.
