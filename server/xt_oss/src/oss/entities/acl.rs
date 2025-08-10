use super::bucket::Owner;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AccessControlList {
    #[serde(rename(deserialize = "Grant"))]
    pub grant: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessControlPolicy {
    #[serde(rename = "Owner")]
    pub owner: Owner,
    #[serde(rename = "AccessControlList")]
    pub access_control_list: AccessControlList,
}
