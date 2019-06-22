Future crate to share kafka client metrics and stats collecting client context:

  1. Move Kafka metrics from `service/tasks` to here.
  2. Move `ClientStatsContext` from `service/tasks` to here.
  3. Register `externals/kafka` metrics once from `bin/replicante`.
  4. Use the `ClientStatsContext` for `stream/stream` kafka backend.
