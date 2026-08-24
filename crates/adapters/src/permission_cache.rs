use std::{sync::Arc, time::Duration};

use adoc_application::permission::{CachedPoint, PermissionCache};
use adoc_ports::BoxFuture;
use redis::aio::ConnectionManager;

const CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct RedisPermissionCache {
    connection: ConnectionManager,
    namespace: Arc<str>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct UnavailablePermissionCache;

impl PermissionCache for UnavailablePermissionCache {
    fn get<'a>(&'a self, _key: &'a str) -> BoxFuture<'a, Result<Option<CachedPoint>, ()>> {
        Box::pin(async { Err(()) })
    }

    fn put<'a>(&'a self, _key: &'a str, _value: &'a CachedPoint) -> BoxFuture<'a, Result<(), ()>> {
        Box::pin(async { Err(()) })
    }
}

impl RedisPermissionCache {
    pub async fn connect(url: &str, namespace: &str) -> Result<Self, ()> {
        if namespace.is_empty()
            || namespace.len() > 100
            || !namespace
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(());
        }
        let client = redis::Client::open(url).map_err(|_| ())?;
        let connection =
            tokio::time::timeout(Duration::from_secs(3), client.get_connection_manager())
                .await
                .map_err(|_| ())?
                .map_err(|_| ())?;
        Ok(Self {
            connection,
            namespace: Arc::from(namespace),
        })
    }

    fn namespaced(&self, key: &str) -> String {
        format!("{}:{key}", self.namespace)
    }
}

impl PermissionCache for RedisPermissionCache {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<CachedPoint>, ()>> {
        Box::pin(async move {
            let mut connection = self.connection.clone();
            let value: Option<String> = tokio::time::timeout(
                Duration::from_millis(200),
                redis::cmd("GET")
                    .arg(self.namespaced(key))
                    .query_async(&mut connection),
            )
            .await
            .map_err(|_| ())?
            .map_err(|_| ())?;
            value
                .map(|value| serde_json::from_str(&value).map_err(|_| ()))
                .transpose()
        })
    }

    fn put<'a>(&'a self, key: &'a str, value: &'a CachedPoint) -> BoxFuture<'a, Result<(), ()>> {
        Box::pin(async move {
            let value = serde_json::to_string(value).map_err(|_| ())?;
            let mut connection = self.connection.clone();
            let _: String = tokio::time::timeout(
                Duration::from_millis(200),
                redis::cmd("SET")
                    .arg(self.namespaced(key))
                    .arg(value)
                    .arg("EX")
                    .arg(CACHE_TTL.as_secs())
                    .query_async(&mut connection),
            )
            .await
            .map_err(|_| ())?
            .map_err(|_| ())?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn namespace_rejects_cache_key_injection() {
        assert!(
            "adoc-prod"
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        );
        assert!(
            !"adoc:other"
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        );
    }
}
