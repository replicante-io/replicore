## Data-store
A software solution specialised in storing data for other software.
The term datastore is used to avoid focus on
details of specific storage models or uses.

For the context of this project a datastore is more generically defined
as a software that fullfils the requirements of the [model](model/README.md).


## Model
A conceptual representation of an abstract system that detiles
and defines all its properties.

Wikipedia definition: https://en.wikipedia.org/wiki/Conceptual_model


## Node
An instance of the datastore, usually running on
different (vitual or pythsical) machines.


## Primary
The node in the cluster that is responsible
for coordination of a specific shard.


## Secondary
A node in the cluster that is responsible for keeping a copy of a shard.


## Shard
A subset of the data in the datastore.
Each shard is managed and operated on indepently of other.
Shards usually also fail independently.
