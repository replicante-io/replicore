# Deprecate #[macro_use]
Status: PENDING
Reason: As of rust 2018 exact macros can be imported.
Blocking something: NO


## Task
Importing macros explicitly makes things much clearer.

  * For each crate:
    * For each `#[macro_use]` import:
      * Remove the `macro_use` from `lib.rs`.
      * Fix all tests by importing needed macros.
