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

#[tokio::test]
async fn test_invalid_system_settings_update_is_atomic() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let resp = TestClient::put(helpers::get_url("/api/admin/system-settings"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "storefront_title": "Must Not Persist",
            "distribution_enabled": true,
            "distribution_default_rate_bps": 10001
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "invalid_atomic_settings_update").await;
    assert!(!json["success"].as_bool().unwrap());

    let resp = TestClient::get(helpers::get_url("/api/site-settings"))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "settings_after_invalid_update").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["storefront_title"], "LicenseHub");
    assert_eq!(json["data"]["distribution"]["enabled"], false);
    assert_eq!(json["data"]["distribution"]["default_rate_bps"], 2000);
}

#[tokio::test]
async fn test_generate_license_signing_key_returns_admin_keys_only() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let resp = TestClient::get(helpers::get_url("/api/admin/system-settings"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "system_settings_before_key").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["license_signing"]["configured"], false);
    assert!(json["data"]["license_signing"]["public_key_b64"].is_null());
    assert!(json["data"]["license_signing"]["private_key_b64"].is_null());

    let resp = TestClient::post(helpers::get_url("/api/admin/system-settings/license-key"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({"rotate": false}))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "generate_license_key").await;
    assert!(json["success"].as_bool().unwrap());

    let license = &json["data"]["license_signing"];
    assert_eq!(license["configured"], true);
    assert_eq!(license["key_id"].as_str(), Some("license-v1"));
    assert_eq!(
        license["public_key_b64"].as_str().unwrap_or_default().len(),
        43
    );
    assert_eq!(
        license["private_key_b64"]
            .as_str()
            .unwrap_or_default()
            .len(),
        43
    );
    assert!(license["updated_at"].as_str().is_some());

    let resp = TestClient::get(helpers::get_url("/api/site-settings"))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "public_site_settings_after_key").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(
        json["data"]["license_signing"]["public_key_b64"]
            .as_str()
            .is_some()
    );
    assert!(json["data"]["license_signing"]["private_key_b64"].is_null());
}
