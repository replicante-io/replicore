# Rewrite codebase to be async
Status: BLOCKED (see timeline below)
Reason: Async code is awesome and would likey simplify some concepts in the codebase.
Blocking something: YES (rewrite-tasks)


## Task
Refactor replicante core codebase to be asynchronous, based on futures and tokio.

Many dependencies in use are already async and the recent introductions to rust (async/await & futues)
make async code almost as easy as "sync" code, while delivering the greater power,
performance and flexibility of async code.

I've been wanting to convert replicante to async and tokio for a while now and the time is fast approaching.


## Steps
This task can be approached bottom-up (convert low-level components first, and block_on in callers)
as well as top-down (convert from main function and spawn_blocking non-async calls it makes).

I have chosen the top-down approach because:

  * It would allow to address the "global" design early on.
  * Feels less risky (spawn_blocking is a runtime proveded feature, block_on is more mysterious to me).
  * Once available, entriely new components can be async/await from the start.

Current idea

   1. Initialise the runtime and convert the main function
      * Initialise tokio engine.
      * Initialise actix-web runtime to use the tokio engine.
      * Block on spawining the existing system with `spawn_blocking`.
   2. Solve the graceful shutdown problem
      * This needs some signal to initiate (`tokio-signal`).
      * A way to propagate it.
      * And a way to wait, up to a timeout, for the signalled elements to act.
   3. Solve the early shutdown problem (when components crash or panic main should exit).
   4. Remove `humthreads` (as step 2 should solve the problem in its place and I don't like it).
   5. Refactor how components are spawned, making replicante core init function async.
   6. Enumerate components that needs to be converted and what interfaces they depend on.
   7. Rank components by complexity (including number of interfaces they require).
   8. Evaluation time:
      * Should now switch to bottom-up where interfaces are made asyncs before components (how does waiting work)?
      * How can interface be "made async" while remainging blocking (`spawn_blocking` everything)?
      * Implement parallel async interfaces to migrate to, the remove the sync interfaces when no longer used?
        * This could allow re-working crates organization into `<interface>/{base,$impl1,$impl2,...}`.
   9. Convert components/interfaces, one at a time.
  10. Convert interfaces/components, one at a time.
  11. Once replicante core is fully converted, move on to agents.


## Timeline
This is an "awesome to have" but Replicante needs features right now.
Specifically I want the following features before focusing on a fully async codebase:

  * Declarative clusters.
  * Datastore connection encryption (where supported by datastore).
  * Certificate management support (cert rollout and rotation; supported by open source CAs).

To ensure we remain as focused as possible on these goals:

  * Steps 1 to 3 are "blocked" by declarative clusters.
  * Steps 4 to 8 are "blocked" by datastore connection encryption.
  * Steps 9 and 10 are "blocked" by certificate management support (depending on complexity).
