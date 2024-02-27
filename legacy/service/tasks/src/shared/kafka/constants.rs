pub static KAFKA_ADMIN_CONSUMER: &str = "replicante.tasks.admin";
pub static KAFKA_ADMIN_GROUP: &str = "replicante.tasks.admin";

pub static KAFKA_MESSAGE_QUEUE_MIN: &str = "5";
pub static KAFKA_STATS_INTERVAL: &str = "1000";

pub static KAFKA_CLIENT_ID_CONSUMER: &str = "replicante.tasks.workers";
pub static KAFKA_CLIENT_ID_TASKS_PRODUCER: &str = "replicante.tasks.requester";
pub static KAFKA_CLIENT_ID_RETRY_PRODUCER: &str = "replicante.tasks.retrier";

pub static KAFKA_TASKS_GROUP: &str = "replicante.tasks.worker";
pub static KAFKA_TASKS_ID_HEADER: &str = "meta:task:id";
pub static KAFKA_TASKS_RETRY_HEADER: &str = "meta:task:retry";
