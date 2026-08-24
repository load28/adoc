use std::{sync::Arc, time::Duration};

use adoc_application::identity::{IdentityError, LoginRateLimitScope, LoginRateLimiter, TokenHash};
use adoc_ports::BoxFuture;
use redis::aio::ConnectionManager;

const WINDOW: Duration = Duration::from_secs(600);
const START_LIMIT: u32 = 20;
const CALLBACK_LIMIT: u32 = 40;
const ATOMIC_FIXED_WINDOW: &str = r#"
local blocked = 0
for _, key in ipairs(KEYS) do
  local count = redis.call('INCR', key)
  if count == 1 then
    redis.call('EXPIRE', key, ARGV[1])
  end
  if count > tonumber(ARGV[2]) then
    blocked = 1
  end
end
return blocked
"#;

#[derive(Clone)]
pub struct RedisLoginRateLimiter {
    connection: ConnectionManager,
    namespace: Arc<str>,
}

impl RedisLoginRateLimiter {
    pub async fn connect(url: &str, namespace: &str) -> Result<Self, IdentityError> {
        if namespace.is_empty()
            || namespace.len() > 100
            || !namespace
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(IdentityError::Internal);
        }
        let client = redis::Client::open(url).map_err(|_| IdentityError::StorageUnavailable)?;
        let connection =
            tokio::time::timeout(Duration::from_secs(3), client.get_connection_manager())
                .await
                .map_err(|_| IdentityError::StorageUnavailable)?
                .map_err(|_| IdentityError::StorageUnavailable)?;
        Ok(Self {
            connection,
            namespace: Arc::from(namespace),
        })
    }

    fn key(&self, scope: LoginRateLimitScope, signal: &TokenHash) -> String {
        let scope = match scope {
            LoginRateLimitScope::Start => "start",
            LoginRateLimitScope::Callback => "callback",
        };
        format!(
            "{}:auth-rate:{scope}:{}",
            self.namespace,
            hex::encode(signal.0)
        )
    }
}

impl LoginRateLimiter for RedisLoginRateLimiter {
    fn check<'a>(
        &'a self,
        scope: LoginRateLimitScope,
        signals: Vec<TokenHash>,
    ) -> BoxFuture<'a, Result<(), IdentityError>> {
        Box::pin(async move {
            if signals.is_empty() {
                return Err(IdentityError::Internal);
            }
            let keys = signals
                .iter()
                .map(|signal| self.key(scope, signal))
                .collect::<Vec<_>>();
            let mut command = redis::cmd("EVAL");
            command
                .arg(ATOMIC_FIXED_WINDOW)
                .arg(signals.len())
                .arg(&keys)
                .arg(WINDOW.as_secs())
                .arg(match scope {
                    LoginRateLimitScope::Start => START_LIMIT,
                    LoginRateLimitScope::Callback => CALLBACK_LIMIT,
                });
            let mut connection = self.connection.clone();
            let blocked: i64 =
                tokio::time::timeout(Duration::from_secs(2), command.query_async(&mut connection))
                    .await
                    .map_err(|_| IdentityError::StorageUnavailable)?
                    .map_err(|_| IdentityError::StorageUnavailable)?;
            if blocked == 0 {
                Ok(())
            } else {
                Err(IdentityError::RateLimited)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn rate_limit_keys_contain_only_namespace_scope_and_hash() {
        let key = format!("adoc:auth-rate:start:{}", hex::encode([7_u8; 32]));
        assert_eq!(key.len(), "adoc:auth-rate:start:".len() + 64);
        assert!(!key.contains("192.0.2.10"));
    }
}
