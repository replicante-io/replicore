use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::KafkaConfig;
use crate::traits::StreamInterface;
use crate::Result;
use crate::StreamOpts;

mod kafka;

pub fn kafka<T>(config: KafkaConfig, opts: StreamOpts) -> Result<Arc<dyn StreamInterface<T>>>
where
    T: DeserializeOwned + Serialize + 'static,
{
    let stream = self::kafka::KafkaStream::new(config, opts)?;
    Ok(Arc::new(stream))
}
