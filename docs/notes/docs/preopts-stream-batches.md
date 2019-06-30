---
id: stream-batches
title: Stream Batches
sidebar_label: Stream Batches
---

The current base `Stream` requires each message to be acknowledged before the next one can be fetched.
This is done to ensure the application processes messages in order and does not skip any by mistake.

The downside is that batch processing of messages is not possible.
For example, the event indexer has to read from the stream and commit to the store each message one at a time.
A more efficient approach would be to collect a number of events and insert them in the DB at once.
This would batch both reads and writes from the two systems and be significantly more effificient.


## Possible implementation

  1. Require the stream to commit all past messages when acking one.
  2. Add a `Message::batch(self)` method to allow the stream to proceed.
  3. For kafka, track the max offset for each topic/parition and store them all at the next `async_ack`.
  4. Figure out how to re-process the entire batch: re-creating the iterator may be enough. 


## Why wait?

  * This is not needed now.
  * There are likely more usefull optimisations to look at.
  * It is a simple enough change (at least in kafka) so it can be delayed to when needed.
