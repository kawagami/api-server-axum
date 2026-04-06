pub mod local;
use axum::body::Bytes;
use futures_util::Stream;
pub use local::{LocalStorage, LocalStorageError};

pub enum Storage {
    Local(LocalStorage),
    // Firebase(FirebaseStorage),
}

impl Storage {
    pub fn from_env() -> Self {
        let base_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());
        let base_url = std::env::var("UPLOAD_BASE_URL")
            .unwrap_or_else(|_| "https://kawa.homes/uploads".to_string());

        Storage::Local(LocalStorage::new(&base_path, &base_url))
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
        match self {
            Storage::Local(s) => s.upload(stream, content_type).await,
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), LocalStorageError> {
        match self {
            Storage::Local(s) => s.delete(key).await,
        }
    }
}
