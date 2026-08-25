use crate::types::{
    DeleteRequest, DownloadRequest, DownloadResult, StorageAdapter, StorageError, UploadRequest,
    UploadResult,
};
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

    async fn download(
        &self,
        _request: DownloadRequest<'_>,
    ) -> Result<DownloadResult, StorageError> {
        Err(StorageError::Request(
            "mock storage does not contain downloadable objects".to_string(),
        ))
    }

    async fn delete(&self, _request: DeleteRequest<'_>) -> Result<(), StorageError> {
        Ok(())
    }
}
