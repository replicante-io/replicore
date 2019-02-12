---
id: version-0.1.0-dynamic-config
title: Dynamic configuration
original_id: dynamic-config
---

Whenever a configuration option is used in a "one shot" fashion (the timeout for an operation
is fixed at the start of each operation VS the database server requires a restart to change)
it could be dynamically fetched instead of requiring a process restart.

This would give greater configuration control without restarts as well as provide a central
configuration place (no need to update every configuration file on every server).


## Downsides:

  * Extra complexity in the code.
  * Extra deployment complexity (need a configuration store of sorts).
  * Less obvious configuration methods (upload configuration file to store? Changes done with `replictl`?)


## Possible implementation

  1. Split the configuration object into two static objects:
     * Static configuration options are left in the existing file.
     * Dynamic configuration options are moved to a new file.
  2. Change the dynamic configuration object to have a more dynamic API.
     * Possibly implement some wrapper type that `DeRef` to the requested type and is
       loaded when accessed (possibly cache in memory for sanity).
     * Possibly not so the cost/risk of dynamic lookup can be exposed.
     * Consider a dynamic configuration loader that returns a static view on the full
       dynamic configuration options vs dynamic loading of individual attributes.
     * Keep the stored format decoupled from the exposed interface (mostly store the full
       configuration should be stored and picked from by the API).
     * Provide an official way to update dynamic configuration (`replictl`).
  3. Update user code, if needed, to use a dynamic fetch API.
