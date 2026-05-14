use crate::types::{PaymentAdapter, PaymentError, PaymentMethod};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct PaymentRegistry {
    adapters: BTreeMap<String, Arc<dyn PaymentAdapter>>,
}

impl PaymentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<A>(&mut self, adapter: A)
    where
        A: PaymentAdapter + 'static,
    {
        self.adapters
            .insert(adapter.pay_type().to_string(), Arc::new(adapter));
    }

    pub fn get(&self, pay_type: &str) -> Result<Arc<dyn PaymentAdapter>, PaymentError> {
        self.adapters
            .get(pay_type)
            .cloned()
            .ok_or_else(|| PaymentError::Unsupported(pay_type.to_string()))
    }

    pub fn methods(&self) -> Vec<PaymentMethod> {
        self.adapters
            .values()
            .map(|adapter| PaymentMethod {
                pay_type: adapter.pay_type().to_string(),
                label: adapter.label().to_string(),
                provider: adapter.provider().to_string(),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.adapters.is_empty()
    }
}
