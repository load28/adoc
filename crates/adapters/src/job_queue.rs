use std::time::Duration;

use adoc_application::{
    governance::GovernanceError,
    jobs::JobSignalQueue,
    operations::{JobPriorityBucket, JobSignal, StreamWake},
};
use adoc_ports::BoxFuture;
use futures_util::StreamExt;
use redis::aio::ConnectionManager;
use tokio::sync::broadcast;
use uuid::Uuid;

const STREAM_WAKE_CHANNEL: &str = "adoc:stream:wake:v1";

#[derive(Clone)]
pub struct RedisJobSignalQueue {
    connection: ConnectionManager,
    namespace: String,
}

impl RedisJobSignalQueue {
    pub async fn connect(url: &str, namespace: &str) -> Result<Self, GovernanceError> {
        if namespace.is_empty()
            || namespace.len() > 64
            || !namespace
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(GovernanceError::Validation);
        }
        let client =
            redis::Client::open(url).map_err(|_| GovernanceError::DependencyUnavailable)?;
        let connection = ConnectionManager::new(client)
            .await
            .map_err(|_| GovernanceError::DependencyUnavailable)?;
        Ok(Self {
            connection,
            namespace: namespace.to_owned(),
        })
    }

    fn key(&self, bucket: JobPriorityBucket) -> String {
        let bucket = match bucket {
            JobPriorityBucket::Interactive => "interactive",
            JobPriorityBucket::Normal => "normal",
            JobPriorityBucket::Background => "background",
        };
        format!("adoc:{}:jobs:{bucket}:v1", self.namespace)
    }
}

impl JobSignalQueue for RedisJobSignalQueue {
    fn signal<'a>(&'a self, jobs: &'a [JobSignal]) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let mut connection = self.connection.clone();
            for job in jobs {
                let key = self.key(job.bucket);
                let _: i64 = redis::cmd("LPUSH")
                    .arg(&key)
                    .arg(job.id.to_string())
                    .query_async(&mut connection)
                    .await
                    .map_err(|_| GovernanceError::DependencyUnavailable)?;
                let _: String = redis::cmd("LTRIM")
                    .arg(&key)
                    .arg(0)
                    .arg(9_999)
                    .query_async(&mut connection)
                    .await
                    .map_err(|_| GovernanceError::DependencyUnavailable)?;
            }
            Ok(())
        })
    }

    fn drain<'a>(&'a self, limit: usize) -> BoxFuture<'a, Result<Vec<Uuid>, GovernanceError>> {
        Box::pin(async move {
            if !(1..=1000).contains(&limit) {
                return Err(GovernanceError::Validation);
            }
            let mut connection = self.connection.clone();
            let keys = [
                self.key(JobPriorityBucket::Interactive),
                self.key(JobPriorityBucket::Normal),
                self.key(JobPriorityBucket::Background),
            ];
            let mut ids = Vec::with_capacity(limit);
            while ids.len() < limit {
                let mut found = false;
                for key in &keys {
                    let value: Option<String> = redis::cmd("RPOP")
                        .arg(key)
                        .query_async(&mut connection)
                        .await
                        .map_err(|_| GovernanceError::DependencyUnavailable)?;
                    if let Some(value) = value {
                        found = true;
                        if let Ok(id) = Uuid::parse_str(&value) {
                            ids.push(id);
                        }
                        if ids.len() == limit {
                            break;
                        }
                    }
                }
                if !found {
                    break;
                }
            }
            Ok(ids)
        })
    }

    fn publish_stream_wake<'a>(
        &'a self,
        wake: StreamWake,
    ) -> BoxFuture<'a, Result<(), GovernanceError>> {
        Box::pin(async move {
            let payload = serde_json::to_string(&wake).map_err(|_| GovernanceError::Internal)?;
            let mut connection = self.connection.clone();
            let _: i64 = redis::cmd("PUBLISH")
                .arg(STREAM_WAKE_CHANNEL)
                .arg(payload)
                .query_async(&mut connection)
                .await
                .map_err(|_| GovernanceError::DependencyUnavailable)?;
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct RedisStreamWakeHub {
    sender: broadcast::Sender<StreamWake>,
}

impl RedisStreamWakeHub {
    pub fn start(url: &str, capacity: usize) -> Result<Self, GovernanceError> {
        if !(16..=4096).contains(&capacity) {
            return Err(GovernanceError::Validation);
        }
        let client =
            redis::Client::open(url).map_err(|_| GovernanceError::DependencyUnavailable)?;
        let (sender, _) = broadcast::channel(capacity);
        let task_sender = sender.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(mut pubsub) = client.get_async_pubsub().await
                    && pubsub.subscribe(STREAM_WAKE_CHANNEL).await.is_ok()
                {
                    let mut messages = pubsub.on_message();
                    while let Some(message) = messages.next().await {
                        if let Ok(payload) = message.get_payload::<String>()
                            && let Ok(wake) = serde_json::from_str::<StreamWake>(&payload)
                        {
                            let _ = task_sender.send(wake);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        Ok(Self { sender })
    }

    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<StreamWake> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bounded_wake_hub_reports_slow_consumers() {
        let (sender, _) = broadcast::channel(16);
        let hub = RedisStreamWakeHub { sender };
        let mut receiver = hub.subscribe();
        for sequence in 1..=17 {
            hub.sender
                .send(StreamWake {
                    workspace_id: Uuid::nil(),
                    sequence,
                })
                .unwrap();
        }
        assert!(matches!(
            receiver.recv().await,
            Err(broadcast::error::RecvError::Lagged(1))
        ));
    }
}
