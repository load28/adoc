use std::{
    io::ErrorKind,
    path::{Path, PathBuf},
};

use adoc_application::operations::{
    ByteRange, ByteStream, MalwareScanner, ObjectMetadata, ObjectStorage, ScanVerdict, StorageError,
};
use adoc_ports::BoxFuture;
use futures_util::{StreamExt, TryStreamExt};
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom},
};
use tokio_util::io::ReaderStream;

#[derive(Clone)]
pub struct LocalObjectStorage {
    root: PathBuf,
}
impl LocalObjectStorage {
    pub fn new(root: PathBuf) -> Result<Self, StorageError> {
        if !root.is_absolute() {
            return Err(StorageError::Unavailable);
        }
        Ok(Self { root })
    }
    fn path(&self, key: &str) -> Result<PathBuf, StorageError> {
        if key.len() != 64
            || !key
                .bytes()
                .all(|v| v.is_ascii_hexdigit() && !v.is_ascii_uppercase())
        {
            return Err(StorageError::Unavailable);
        }
        Ok(self.root.join(&key[..2]).join(&key[2..4]).join(key))
    }
}
impl ObjectStorage for LocalObjectStorage {
    fn write<'a>(
        &'a self,
        key: &'a str,
        mut stream: ByteStream,
        max_bytes: u64,
    ) -> BoxFuture<'a, Result<ObjectMetadata, StorageError>> {
        Box::pin(async move {
            let path = self.path(key)?;
            if fs::try_exists(&path)
                .await
                .map_err(|_| StorageError::Unavailable)?
            {
                return Err(StorageError::AlreadyExists);
            }
            let parent = path.parent().ok_or(StorageError::Unavailable)?;
            fs::create_dir_all(parent)
                .await
                .map_err(|_| StorageError::Unavailable)?;
            let partial = path.with_extension("partial");
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&partial)
                .await
                .map_err(|error| {
                    if error.kind() == ErrorKind::AlreadyExists {
                        StorageError::AlreadyExists
                    } else {
                        StorageError::Unavailable
                    }
                })?;
            let result = async {
                let mut size = 0_u64;
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    size = size
                        .checked_add(chunk.len() as u64)
                        .ok_or(StorageError::Unavailable)?;
                    if size > max_bytes {
                        return Err(StorageError::Unavailable);
                    }
                    file.write_all(&chunk)
                        .await
                        .map_err(|_| StorageError::Unavailable)?
                }
                file.sync_all()
                    .await
                    .map_err(|_| StorageError::Unavailable)?;
                fs::rename(&partial, &path)
                    .await
                    .map_err(|_| StorageError::Unavailable)?;
                sync_dir(parent).await?;
                Ok(ObjectMetadata { size })
            }
            .await;
            if result.is_err() {
                let _ = fs::remove_file(&partial).await;
            }
            result
        })
    }
    fn stat<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<ObjectMetadata, StorageError>> {
        Box::pin(async move {
            let meta = fs::metadata(self.path(key)?).await.map_err(map_io)?;
            Ok(ObjectMetadata { size: meta.len() })
        })
    }
    fn read<'a>(
        &'a self,
        key: &'a str,
        range: Option<ByteRange>,
    ) -> BoxFuture<'a, Result<ByteStream, StorageError>> {
        Box::pin(async move {
            let mut file = File::open(self.path(key)?).await.map_err(map_io)?;
            let stream: ByteStream = if let Some(range) = range {
                file.seek(SeekFrom::Start(range.start))
                    .await
                    .map_err(|_| StorageError::Unavailable)?;
                Box::pin(
                    ReaderStream::new(file.take(range.len()))
                        .map_err(|_| StorageError::Unavailable),
                )
            } else {
                Box::pin(ReaderStream::new(file).map_err(|_| StorageError::Unavailable))
            };
            Ok(stream)
        })
    }
    fn delete<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            match fs::remove_file(self.path(key)?).await {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
                Err(_) => Err(StorageError::Unavailable),
            }
        })
    }
}
async fn sync_dir(path: &Path) -> Result<(), StorageError> {
    File::open(path)
        .await
        .map_err(|_| StorageError::Unavailable)?
        .sync_all()
        .await
        .map_err(|_| StorageError::Unavailable)
}
fn map_io(error: std::io::Error) -> StorageError {
    if error.kind() == ErrorKind::NotFound {
        StorageError::NotFound
    } else {
        StorageError::Unavailable
    }
}

#[derive(Clone, Default)]
pub struct EicarMalwareScanner;
impl MalwareScanner for EicarMalwareScanner {
    fn scan<'a>(
        &'a self,
        mut stream: ByteStream,
    ) -> BoxFuture<'a, Result<ScanVerdict, StorageError>> {
        Box::pin(async move {
            const SIGNATURE: &[u8] =
                b"X5O!P%@AP[4\\PZX54(P^)7CC)7}$EICAR-STANDARD-ANTIVIRUS-TEST-FILE!$H+H*";
            let mut tail = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                tail.extend_from_slice(&chunk);
                if tail.windows(SIGNATURE.len()).any(|part| part == SIGNATURE) {
                    return Ok(ScanVerdict::Malware);
                }
                if tail.len() > SIGNATURE.len() {
                    tail.drain(..tail.len() - SIGNATURE.len());
                }
            }
            Ok(ScanVerdict::Clean)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use futures_util::stream;
    use uuid::Uuid;
    #[tokio::test]
    async fn local_adapter_rejects_traversal_and_supports_range() {
        let root = std::env::temp_dir().join(format!("adoc-object-{}", Uuid::now_v7()));
        fs::create_dir_all(&root).await.unwrap();
        let storage = LocalObjectStorage::new(root.clone()).unwrap();
        assert!(storage.stat("../bad").await.is_err());
        let key = "a".repeat(64);
        storage
            .write(
                &key,
                Box::pin(stream::iter([Ok(Bytes::from_static(b"abcdef"))])),
                6,
            )
            .await
            .unwrap();
        let bytes = storage
            .read(
                &key,
                Some(ByteRange {
                    start: 2,
                    end_inclusive: 4,
                }),
            )
            .await
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap();
        assert_eq!(bytes.concat(), b"cde");
        assert!(matches!(
            storage.write(&key, Box::pin(stream::empty()), 0).await,
            Err(StorageError::AlreadyExists)
        ));
        storage.delete(&key).await.unwrap();
        storage.delete(&key).await.unwrap();
        fs::remove_dir_all(root).await.unwrap();
    }
}
