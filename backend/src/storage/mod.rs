pub mod local;
pub use local::{LocalStorage, LocalStorageError};

pub enum Storage {
    Local(LocalStorage),
    // Firebase(FirebaseStorage),
}

impl Storage {
    pub fn from_env() -> Self {
        let base_path = std::env::var("UPLOAD_PATH").unwrap_or_else(|_| "./uploads".to_string());
        Storage::Local(LocalStorage::new(&base_path))
    }

    /// 寫入已處理完成的檔案 bytes，回傳 (storage_key, 公開 url)。
    /// 內容驗證/轉檔是 caller（services/images.rs）的責任，storage 只管落地。
    pub async fn upload(
        &self,
        data: &[u8],
        ext: &str,
        base_url: &str,
    ) -> Result<(String, String), LocalStorageError> {
        match self {
            Storage::Local(s) => s.upload(data, ext, base_url).await,
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), LocalStorageError> {
        match self {
            Storage::Local(s) => s.delete(key).await,
        }
    }
}
