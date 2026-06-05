use crate::types::{StorageAdapter, StorageError, UploadRequest, UploadResult};
use async_trait::async_trait;

#[derive(Default)]
pub struct MockStorageAdapter;

#[async_trait]
impl StorageAdapter for MockStorageAdapter {
    fn provider(&self) -> &'static str {
        crate::types::PROVIDER_MOCK
    }

    fn label(&self) -> &'static str {
        "Mock Storage"
    }

    async fn upload(&self, _request: UploadRequest<'_>) -> Result<UploadResult, StorageError> {
        Ok(UploadResult {
            etag: Some("mock-etag".to_string()),
        })
    }
}
