# Circuit Breakers
Status: BLOCKED (premature; async)
Reason: Improve resiliancy to failed dependencies.
Blocking something: NO


## Blocked by

  * Need for more features (same as async-rewrite.md).
  * Async will re-write all interfaces and their implementations.
  * Async will make this easier (especially timeouts).


## Task

  * Enumerate dependencies.
  * Prioratise them based on risk/impact.
  * Look at frameworks and libraries for this.
  * For each dependency:
    * Determine what a circuit-broken operation should return.
    * Document, at least, generic read, generic write, special cases.
    * Determine and document how to check if the dependency is back.
    * Implement circuit-breaking for the dependency.
