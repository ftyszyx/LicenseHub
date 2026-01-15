use crate::core::constants::*;
use salvo::{http::StatusCode, prelude::*};
use salvo_oapi::ToSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
    pub success: bool,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: APP_OK,
            message: "success".to_string(),
            data: Some(data),
            success: true,
        }
    }

    #[allow(dead_code)]
    pub fn error_with_message(message: String) -> Self {
        Self {
            code: APP_OTHER,
            message,
            data: None,
            success: false,
        }
    }

    pub fn error_with_message_and_code(message: String, code: u16) -> Self {
        Self {
            code,
            message,
            data: None,
            success: false,
        }
    }
}

#[salvo::async_trait]
impl<T> Writer for ApiResponse<T>
where
    T: Serialize + Send + Sync + 'static,
{
    async fn write(self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.status_code = Some(StatusCode::OK);
        res.render(Json(self));
    }
}

impl<T> salvo_oapi::EndpointOutRegister for ApiResponse<T>
where
    T: salvo_oapi::ToSchema + 'static,
{
    fn register(components: &mut salvo_oapi::Components, operation: &mut salvo_oapi::Operation) {
        operation.responses.insert(
            "200",
            salvo_oapi::Response::new("Ok").add_content(
                "application/json",
                <Self as salvo_oapi::ToSchema>::to_schema(components),
            ),
        );
    }
}
