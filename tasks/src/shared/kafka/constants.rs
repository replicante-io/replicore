pub static KAFKA_ADMIN_CONSUMER: &'static str = "replicante.tasks.admin";
pub static KAFKA_ADMIN_GROUP: &'static str = "replicante.tasks.admin";

pub static KAFKA_MESSAGE_QUEUE_MIN: &'static str = "5";
pub static KAFKA_STATS_INTERVAL: &'static str = "1000";

pub static KAFKA_TASKS_CONSUMER: &'static str = "replicante.tasks.worker";
pub static KAFKA_TASKS_GROUP: &'static str = "replicante.tasks.worker";
pub static KAFKA_TASKS_ID_HEADER: &'static str = "meta:task:id";
pub static KAFKA_TASKS_PRODUCER: &'static str = "replicante.tasks.producer";
pub static KAFKA_TASKS_RETRY_HEADER: &'static str = "meta:task:retry";
pub static KAFKA_TASKS_RETRY_PRODUCER: &'static str = "replicante.tasks.retry.producer";
