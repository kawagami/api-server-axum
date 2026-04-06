use axum::body::Bytes;
use futures_util::{Stream, TryStreamExt};
use std::{io, path::PathBuf, pin::pin};
use tokio::{fs, fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use uuid::Uuid;

pub struct LocalStorage {
    pub base_path: PathBuf,
    pub base_url: String,
}

impl LocalStorage {
    pub fn new(base_path: &str, base_url: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            base_url: base_url.to_string(),
        }
    }

    pub async fn upload<S, E>(
        &self,
        stream: S,
        content_type: &str,
    ) -> Result<(String, String), LocalStorageError>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<axum::BoxError>,
    {
        let ext = ext_from_content_type(content_type);
        let key = format!("{}.{}", Uuid::new_v4(), ext);

        if !key_is_valid(&key) {
            return Err(LocalStorageError::InvalidKey(key));
        }

        fs::create_dir_all(&self.base_path).await?;

        let path = self.base_path.join(&key);

        // stream 直接寫入檔案，不先載入記憶體
        let body_with_io_error = stream.map_err(|e| io::Error::other(e.into()));
        let mut body_reader = pin!(StreamReader::new(body_with_io_error));
        let mut file = BufWriter::new(File::create(&path).await?);
        tokio::io::copy(&mut body_reader, &mut file).await?;

        let url = format!("{}/{}", self.base_url, key);
        Ok((key, url))
    }

    pub async fn delete(&self, key: &str) -> Result<(), LocalStorageError> {
        if !key_is_valid(key) {
            return Err(LocalStorageError::InvalidKey(key.to_string()));
        }

        let path = self.base_path.join(key);
        fs::remove_file(path).await?;
        Ok(())
    }
}

// 防止 directory traversal 攻擊，確保 key 只有一層路徑
fn key_is_valid(key: &str) -> bool {
    let path = std::path::Path::new(key);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

fn ext_from_content_type(ct: &str) -> &str {
    match ct {
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "jpg",
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LocalStorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}
