# Configuration
Replicante provides a large set of configuration options with reasonable defaults.
This supports common use cases where only a handful of options need attention
as well as advanced setup where the user has the power to fine tune most details.


## Required configuration
Some options do not have reasonable defaults so users will have to set them explicitly:

  * [Agents discovery](../features/discovery.md) (vastly depends on user needs).
  * Storage configuration (specifically: address of the DB server).


## All configuration options
All options are documented in the
[example configuration file](https://github.com/replicante-io/replicante/blob/master/replicante.example.yaml)
at the root of the repo, also shown below.

This file shows all options with their defaults and explains their meaning and available settings.
As mentioned above, common use cases should be able to ignore most options if users are so inclined.

Details of these options are documented in the features they influence.

[import, lang:"yaml"](../../../replicante.example.yaml)
