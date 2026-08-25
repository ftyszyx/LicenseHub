pub mod aliyun;
pub mod mock;
pub mod r2;
pub mod registry;
pub mod types;

pub use aliyun::AliyunOssAdapter;
pub use mock::MockStorageAdapter;
pub use r2::CloudflareR2Adapter;
pub use registry::StorageRegistry;
pub use types::{
    DEFAULT_PREFIX, DeleteRequest, DownloadRequest, DownloadResult, PROVIDER_ALIYUN_OSS,
    PROVIDER_CLOUDFLARE_R2, PROVIDER_MOCK, StorageAdapter, StorageChannelConfig, StorageError,
    UploadRequest, UploadResult, build_object_key, build_public_url, normalize_storage_config,
    normalize_storage_provider, parse_storage_config,
};

pub fn default_registry() -> StorageRegistry {
    let mut registry = StorageRegistry::new();
    registry.register(AliyunOssAdapter);
    registry.register(CloudflareR2Adapter);
    registry.register(MockStorageAdapter);
    registry
}
