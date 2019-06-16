---
id: data-storage-format
title: Data storage format
---

Replicante core stores data in a document store.

The format of the data stored here is strictly defined by the
[application code](https://github.com/replicante-io/replicante/tree/master/data/models/src)
and care must be taken to allow for a zero downtime upgrade path.
This means that changing the data format must be incremental:

  1. Make the change optional while supporting the existing data format.
  2. Release and run the code long enough for all data to be regenerated or aged-out.
  3. Make the new data format a requirement and introduce features that rely on it.
  4. Release the code with the new data format and features.

Each data item is stored and updated atomically.
The encoding details vary from store to store but Replicante uses a general interface
to interact with the store, regardless of the functionality it exposes.


## Why a documents store?
The main reasons for choosing [MongoDB](https://www.mongodb.com/) as the store are:

  * *Flexible document format*: since Replicante is in early development stages the data format
    has not yet being tried and tested.
    Many features that will require (potentially large) changes to the data format are in the
    future of the project as well.
    Having a store with a low overhead to schema changes is of great value at this stage.
  * *MongoDB is quite easy to setup and support*: Replication, rolling restarts and upgrades,
    automated failover are also requirements for an "always on" solution like Replicante that
    are not easily available out of the box in traditional SQL servers.
  * *Scalability*: MongoDB comes in replica set mode for high availability as well as a slightly
    more complex sharded cluster mode.
    Since Replicante logically groups data by cluster (maybe one day by organisation instead?)
    it is possible to shard large collections and grow/parallelise work (although if you get
    to that level already you HAVE to let me know!).


## Why not a transactional store?
The main reason store transactions are not in use is because the desired database (MongoDB)
does not support them.
The reasons MongoDB was the preferred store to begin with are listed above.

Although the current implementation is not making use of transactions,
things may change in the future.
More and more transactional document stores are becoming available and MongoDB itself
recently added support for transactions in non-sharded collections.

Finally the value of transactions is questionable since the data source has no
atomicity guarantee to begin with.
When refreshing the state of each cluster node, all we can say is the data returned by
a single call to an agent endpoint is consistent in itself.
There are no guarantees the result of two calls to the same agent, one after the other,
would return results that are consistent across the two calls.
