use std::{io, path::PathBuf};
use tokio::fs;
use uuid::Uuid;

pub struct LocalStorage {
    pub base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
        }
    }

    pub async fn upload(
        &self,
        data: &[u8],
        ext: &str,
        base_url: &str,
    ) -> Result<(String, String), LocalStorageError> {
        let key = format!("{}.{}", Uuid::new_v4(), ext);

        if !key_is_valid(&key) {
            return Err(LocalStorageError::InvalidKey(key));
        }

        fs::create_dir_all(&self.base_path).await?;
        fs::write(self.base_path.join(&key), data).await?;

        let url = format!("{}/{}", base_url, key);
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

#[derive(Debug, thiserror::Error)]
pub enum LocalStorageError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid key: {0}")]
    InvalidKey(String),
}
