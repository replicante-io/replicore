# Rename Validator interfaces
Status: PENDING
Reason: rename `Validator` interfaces to `Admin` as they grow in features.
Bloking something: NO


## Task
There are some components that use a `Validator` struct to perform `replictl`-related duties.
The struct named `Admin` was introduced later in other components to perform the same tasks plus more.
For consistency, go change the old `Validator` strucs to be named `Admin` too.


## Plan

 1. [ ] Find all the `Validator`s that need to be renamed.
    * [ ] Data store
    * [ ] ???
 2. [ ] Rename one component.
 3. [ ] Fix all the errors to use the new name.
 4. [ ] Repeat 2 and 3 untill all components are renamed.
