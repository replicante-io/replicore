# Addressing warnings
Status: **ONGOING**
Reason: deprecate legacy errors code and address clippy warnings.
Bloking something: NO


## Task
Address all compiler and clippy warnings to improve code quality.

  1. Address use of deprecated errors since the move to `failure` crate.
  2. Address clippy warnings.


## Plan

  1. [ ] Address compiler warnings:
     * Each time a task is complete address at least one warning, ideally all in a file.
     * Continue until all warnings are gone.
     * Replace legacy errors with appropriate `ErrorKind`s.
     * Delete the deprecated code.
  2. [ ] Address clippy warnings about `new` not returning `Self`:
     * Create a new method with different name.
     * Forward `new` calls to the future constructor and deprecate the `new` function.
     * Address all compiler wranings about deprecated `new` by calling the new function.
     * Remove deprecated `new` function.
  3. [ ] Address other clippy warnings:
     * Each time a task is complete address at least one warning, ideally all in a file.
     * Continue until all warnings are gone.
