# Interfaces rewrite
Status: BLOCKED (premature)
Reason: most `Interfaces` are not nice to work with and are inconvenient to carry around.
Blocking something: NO


## Task
Iterate over `Interfaces` object and initialisation to make code nicer to write
and the interfaces more accessible and extendible.

The reality is most components need most interfaces,
adding new ones will be painfult at this time,
and they get cloned individually all the time.

On the other hand I'd really rather avoid many (or any?) singletons.  
And the problem of interfaces needing other interfaces still needs to be addressed.


## Ideas
While nothing is clear yet (makes me sad), the higher priority for now is building
declarative clusters (and possibly introduce tools for datastore security).
So instead of attempting to solve the problem, here is a list of ideas and what they are aimed at:

  * Switch to a builder pattern instead of the current angle.
  * Figure out the builder pattern for the API (where components can inject handlers after the interfaces are defined)
    * Maybe the API interface is not part of `Interfaces`.
    * Maybe a separate API builder is given to interfaces & components initialisation.
  * The builder returns a single `GlobalContext` object to provide access to all interfaces.
  * The `GlobalContext` instance should be cheap to clone and `Sync + Safe`.
  * Interfaces are accessed by reference (and no longer need to be `Clone` themselves?).
  * Possible things to make "tracking" with components/interfaces use which ones?
