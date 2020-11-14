# Compatibility Modes and upgrades
Status: BLOCKED (premature)
Reason: Some additive changes (emitting new events) can cause compatibility issues.
Blocking something: NO


## Task
Introduce the concept of compatibility mode to skip some code in newer versions.
Inspired by MongoDB version compatibility mode, each version would only support one or two modes
to allow for rolling upgrades (earlier version & current version).

Potentially more versions need to be supported as removing a compatibility mode sounds like
a breaking change in semver terms.


## Ideas
  * An interface to wrap code blocks (see below).
  * Current mode is stored in coordinator? or primary store?
  * Current mode is cached in each process to speed up lookups.
  * Minimise the number of modes in the code at any one point.

```rust
compatibility_mode.require(COMPAT_VERSION? SEMVER_MATCHER?, || {
    # Code to execute only if the current compatiblity mode matches.
    return some_value;
})
```


## Timeline
  * There is no rush yet (much more important stuff should be first).
  * This becomes more important as the project becomes used (because updates are then needed).
  * Possibly focus on better supporting rebuilds first?
    * Or is rolling updates more important?
    * Would need to check what user needs are.
