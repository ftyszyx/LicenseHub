use salvo::http::StatusCode;
use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn test_system_settings_storefront_title() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let resp = TestClient::get(helpers::get_url("/api/site-settings"))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "public_site_settings").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["storefront_title"].as_str().unwrap(),
        "LicenseHub"
    );

    let resp = TestClient::put(helpers::get_url("/api/admin/system-settings"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "storefront_title": "慧达电脑科技"
        }))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "update_system_settings").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["storefront_title"].as_str().unwrap(),
        "慧达电脑科技"
    );

    let resp = TestClient::get(helpers::get_url("/api/site-settings"))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "updated_public_site_settings").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["storefront_title"].as_str().unwrap(),
        "慧达电脑科技"
    );
}
