use crate::types::{StorageAdapter, StorageError};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct StorageRegistry {
    adapters: BTreeMap<String, Arc<dyn StorageAdapter>>,
}

impl StorageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, adapter: A)
    where
        A: StorageAdapter + 'static,
    {
        self.adapters
            .insert(adapter.provider().to_string(), Arc::new(adapter));
    }

    pub fn get(&self, provider: &str) -> Result<Arc<dyn StorageAdapter>, StorageError> {
        self.adapters
            .get(provider)
            .cloned()
            .ok_or_else(|| StorageError::Unsupported(provider.to_string()))
    }
}
