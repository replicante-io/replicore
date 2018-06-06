# Rolling upgrades
No software is perfect and few are ever complete.
Updates are, or should be, part of the day to day operations.
This document describes the steps you can follow to upgrade Replicante without service interruption.

**Note**: this document does NOT describe how to update replicante's dependencies.
You should take care to keep them up to date as well.

The upgrade path relies on Replicante high availability features to avoid loss of service.
Nodes are updated one at a time, upgrading them between compatible versions.
Once all nodes are up to date and restarted the upgrade is complete.


  1. Read the [release notes](./notes.md) for any special consideration for the version you are upgrading to.
  2. Obtain a copy of the code to update to (see the [install](../setup/install.md) section).
  3. Test configuration and store contents compatibility (see [`replictl check upgrade`](../replictl/checks.md)).
  4. For each Replicante server:
    1. [Install](../setup/install.md) the new binary.
    2. Restart the process on the node.
    3. Check the logs to ensure everything works as expect.

Once the upgrade is complete you should review the configuration to enable new options
and disable any deprecated features.


## Changes to data schema
Any software that persist data needs to deal with changes to the schema as the software evolves.
Replicante tries to avoid the need for migrations by gradually rolling out changes to the data.

When the data schema needs to change:

  1. A minor version is released to introduce the changes as optional:
    * New attributes are added and always set but expected to be missing.
    * Deprecated attributes are no longer used but still set.
  2. As clusters are updated collected and generated data will have the new schema.
  3. A major version is released expecting the data to have the new schema:
    * New attributes are no longer optional and decoding data without them fails.
    * Deprecated attributes are no longer used or set.


{% hint style='info' %}
**Upgrading to major releases**

Always run the latest minor release before upgrades to the next major release.

The approach above works because replicante expires historical data and refreshes monitoring data.
You may need to run the latest minor release until all data items are expired or refreshed.
The [`replictl check upgrade`](../replictl/checks.md) command can scan your dataset
to ensure all items are compatible with the new format.
{% endhint %}
