-- Store task payload as JSON encoded text and metadata along side it..
-- This is unlike other SQLite store implementations because tasks don't have a natural
-- struct that encapsulates all their attributes.
CREATE TABLE IF NOT EXISTS tasks_queue(
  -- Manually managed columns for task persisting.
  task_id INTEGER PRIMARY KEY AUTOINCREMENT,
  queue_id TEXT NOT NULL,
  payload TEXT NOT NULL,

  -- Columns to track retry metadata 
  --  Tasks start with a number of reties and are dropped when retries reach -1.
  retries INTEGER NOT NULL,
  --  Delay before tasks are re-delivered in case of failures (includes redelivery on panic).
  retry_delay INTEGER NOT NULL,
  --  EPoc timestamp (in seconds) for the next retry attempt.
  --  Null when tasks have never been delivered.
  next_retry INTEGER DEFAULT NULL
);
