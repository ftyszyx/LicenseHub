use crate::core::my_error::AppError;
use crate::utils::convert::from_str_optional;
use serde::{Deserialize, Serialize};

const DEFAULT_PAGE: u64 = 1;
const DEFAULT_PAGE_SIZE: u64 = 20;
pub const MAX_PAGE_SIZE: u64 = 100;

#[derive(Deserialize, Debug, Serialize)]
pub struct ListParamsReq {
    #[serde(deserialize_with = "from_str_optional", default)]
    pub page: Option<u64>,
    #[serde(deserialize_with = "from_str_optional", default)]
    pub page_size: Option<u64>,
}

impl ListParamsReq {
    pub fn resolve(&self) -> Result<(u64, u64), AppError> {
        let page = self.page.unwrap_or(DEFAULT_PAGE);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

        if page == 0 {
            return Err(AppError::validation(
                "page must be greater than or equal to 1",
            ));
        }
        if !(1..=MAX_PAGE_SIZE).contains(&page_size) {
            return Err(AppError::validation(format!(
                "page_size must be between 1 and {}",
                MAX_PAGE_SIZE
            )));
        }
        if (page - 1).checked_mul(page_size).is_none() {
            return Err(AppError::validation("pagination offset is too large"));
        }

        Ok((page, page_size))
    }
}

impl Default for ListParamsReq {
    fn default() -> Self {
        Self {
            page: Some(DEFAULT_PAGE),
            page_size: Some(DEFAULT_PAGE_SIZE),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PagingResponse<T> {
    pub list: Vec<T>,
    pub page: u64,
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pagination_uses_defaults_and_rejects_invalid_boundaries() {
        assert_eq!(ListParamsReq::default().resolve().unwrap(), (1, 20));
        assert_eq!(
            ListParamsReq {
                page: Some(1),
                page_size: Some(MAX_PAGE_SIZE),
            }
            .resolve()
            .unwrap(),
            (1, MAX_PAGE_SIZE)
        );
        assert!(
            ListParamsReq {
                page: Some(0),
                page_size: Some(20),
            }
            .resolve()
            .is_err()
        );
        assert!(
            ListParamsReq {
                page: Some(1),
                page_size: Some(0),
            }
            .resolve()
            .is_err()
        );
        assert!(
            ListParamsReq {
                page: Some(1),
                page_size: Some(MAX_PAGE_SIZE + 1),
            }
            .resolve()
            .is_err()
        );
        assert!(
            ListParamsReq {
                page: Some(u64::MAX),
                page_size: Some(MAX_PAGE_SIZE),
            }
            .resolve()
            .is_err()
        );
    }
}
