use crate::utils::convert::from_str_optional;
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Debug, Serialize)]
pub struct ListParamsReq {
    #[serde(deserialize_with = "from_str_optional", default)]
    pub page: Option<u64>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub page_size: Option<u64>,
}

impl Default for ListParamsReq {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(20),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PagingResponse<T> {
    pub list: Vec<T>,
    pub page: u64,
    pub total: u64,
}
