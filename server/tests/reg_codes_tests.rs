use crate::helpers::print_response_body_get_json;
use app_server::core::constants;
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
mod helpers;

#[tokio::test]
async fn test_validate_reg_code_post_and_get() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let app_key = format!("KEY_{}", chrono::Utc::now().timestamp());
    let create_app_body = json!({
        "name": format!("VA-App-{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.va.{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": app_key,
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "create_app_for_validate").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;
    let code = format!("CODE_{}", chrono::Utc::now().timestamp());
    let create_rc = json!({
        "code": code,
        "app_id": app_id,
        "valid_days": 7,
        "max_devices": 1,
        "status": 0,
        "code_type": 0
    });
    let _ = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_rc)
        .send(&ctx.app)
        .await;
    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code":code, "app_key":app_key, "device_id":"dev-1"}))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/reg/validate?code={}&app_key={}&device_id=dev-1",
        code, app_key
    )))
    .send(&ctx.app)
    .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    print_response_body_get_json(resp, "validate_reg_code_post_and_get").await;
}

#[tokio::test]
async fn test_validate_device_without_code() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    // create app with trial_days
    let app_key = format!("KEY_{}", chrono::Utc::now().timestamp());
    let create_app_body = json!({
        "name": format!("VA-App-{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.va.{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": app_key,
        "trial_days": 7,
        "sort_order": 0,
        "status": 1
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let _ = print_response_body_get_json(resp, "create_app_for_device_only").await;

    // validate without code, only device binding
    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"app_key":app_key, "device_id":"dev-only-1"}))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(resp, "validate_device_only").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["code_type"].as_i64().unwrap(), 0); // Time type
    assert!(json["data"]["expire_time"].is_string());

    // second call should still succeed before expire
    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"app_key":app_key, "device_id":"dev-only-1"}))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
}

#[tokio::test]
async fn test_get_reg_codes_list() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let response = TestClient::get(helpers::get_url(
        "/api/admin/reg_codes/list?page=1&page_size=10",
    ))
    .add_header("authorization", format!("Bearer {}", ctx.token), true)
    .send(&ctx.app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "get_reg_codes_list").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["list"].is_array());
    assert!(json["data"]["total"].is_number());
    assert!(json["data"]["page"].is_number());
}

#[tokio::test]
async fn test_create_reg_code() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let create_app_body = json!({
        "name": format!("TestApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.test.regcode_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "app_update_info": "Test app for reg code testing",
        "sort_order": 1,
        "status": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "create_app_response").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;
    let create_reg_code_body = json!({
        "code": format!("TESTCODE_{}", chrono::Utc::now().timestamp()),
        "app_id": app_id,
        "bind_device_info": {
            "device_type": "android",
            "device_id": "test_device_123"
        },
        "code_type": 0,
        "valid_days": 30,
        "max_devices": 5,
        "status": 0
    });
    let response = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_reg_code_body)
        .send(&ctx.app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "create_reg_code_response").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["id"].is_number());
    assert_eq!(json["data"]["app_id"], app_id);
    assert_eq!(json["data"]["valid_days"], 30);
    assert_eq!(json["data"]["max_devices"], 5);
    assert_eq!(json["data"]["status"], 0);
}

#[tokio::test]
async fn test_update_reg_code() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let create_app_body = json!({
        "name": format!("TestApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.test.update_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "sort_order": 1,
        "status": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_app_for_update").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;
    let create_reg_code_body = json!({
        "code": format!("UPDATETEST_{}", chrono::Utc::now().timestamp()),
        "app_id": app_id,
        "valid_days": 7,
        "max_devices": 3,
        "status": 0,
        "code_type": 0
    });
    let response = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_reg_code_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_reg_code_for_update").await;
    let reg_code_id = json["data"]["id"].as_i64().unwrap();
    let update_reg_code_body = json!({
        "valid_days": 60,
        "max_devices": 10,
        "status": 1,
        "bind_device_info": {"device_type": "ios", "updated": true}
    });
    let response = TestClient::put(helpers::get_url(&format!(
        "/api/admin/reg_codes/{}",
        reg_code_id
    )))
    .add_header("authorization", format!("Bearer {}", ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&update_reg_code_body)
    .send(&ctx.app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "update_reg_code_response").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["valid_days"], 60);
    assert_eq!(json["data"]["max_devices"], 10);
    assert_eq!(json["data"]["status"], 1);
}

#[tokio::test]
async fn test_get_reg_code_by_id() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let create_app_body = json!({
        "name": format!("TestApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.test.getbyid_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "sort_order": 1,
        "status": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_app_for_get_by_id").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;
    let reg_code_text = format!("GETBYID_{}", chrono::Utc::now().timestamp());
    let create_reg_code_body = json!({
        "code": reg_code_text,
        "app_id": app_id,
        "valid_days": 15,
        "max_devices": 2,
        "status": 0,
        "code_type": 0
    });
    let response = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_reg_code_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_reg_code_for_get_by_id").await;
    let reg_code_id = json["data"]["id"].as_i64().unwrap();
    let response = TestClient::get(helpers::get_url(&format!(
        "/api/admin/reg_codes/{}",
        reg_code_id
    )))
    .add_header("authorization", format!("Bearer {}", ctx.token), true)
    .send(&ctx.app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "get_reg_code_by_id_response").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["id"], reg_code_id);
    assert_eq!(json["data"]["code"], reg_code_text);
    assert_eq!(json["data"]["app_id"], app_id);
    assert_eq!(json["data"]["valid_days"], 15);
    assert_eq!(json["data"]["max_devices"], 2);
    assert!(json["data"]["app_name"].is_string());
}

#[tokio::test]
async fn test_delete_reg_code() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let create_app_body = json!({
        "name": format!("TestApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.test.delete_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "sort_order": 1,
        "status": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_app_for_delete").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;
    let create_reg_code_body = json!({
        "code": format!("DELETE_{}", chrono::Utc::now().timestamp()),
        "app_id": app_id,
        "valid_days": 1,
        "max_devices": 1,
        "status": 0,
        "code_type": 0
    });
    let response = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_reg_code_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_reg_code_for_delete").await;
    let reg_code_id = json["data"]["id"].as_i64().unwrap();
    let response = TestClient::delete(helpers::get_url(&format!(
        "/api/admin/reg_codes/{}",
        reg_code_id
    )))
    .add_header("authorization", format!("Bearer {}", ctx.token), true)
    .send(&ctx.app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "delete_reg_code_response").await;
    assert!(json["success"].as_bool().unwrap());
    let response = TestClient::get(helpers::get_url(&format!(
        "/api/admin/reg_codes/{}",
        reg_code_id
    )))
    .add_header("authorization", format!("Bearer {}", ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = print_response_body_get_json(response, "delete_reg_code_response").await;
    assert_eq!(
        json["code"].as_i64().unwrap(),
        constants::APP_NOT_FOUND as i64
    );
}

#[tokio::test]
async fn test_reg_codes_pagination() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let test_cases = vec![
        helpers::get_url("/api/admin/reg_codes/list?page=1&page_size=5"),
        helpers::get_url("/api/admin/reg_codes/list?page=1&page_size=20"),
        helpers::get_url("/api/admin/reg_codes/list"),
    ];
    for url in test_cases {
        let response = TestClient::get(url)
            .add_header("authorization", format!("Bearer {}", ctx.token), true)
            .send(&ctx.app)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::OK));
        let json = print_response_body_get_json(response, "reg_codes_pagination").await;
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"]["list"].is_array());
        assert!(json["data"]["total"].is_number());
        assert!(json["data"]["page"].is_number());
    }
}

#[tokio::test]
async fn test_reg_codes_search_filters() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let test_cases = vec![
        helpers::get_url("/api/admin/reg_codes/list?status=0"),
        helpers::get_url("/api/admin/reg_codes/list?status=1"),
        helpers::get_url("/api/admin/reg_codes/list?code=TEST"),
        helpers::get_url("/api/admin/reg_codes/list?app_id=1"),
    ];
    for url in test_cases {
        let response = TestClient::get(url)
            .add_header("authorization", format!("Bearer {}", ctx.token), true)
            .send(&ctx.app)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::OK));
        let json = print_response_body_get_json(response, "reg_codes_filters").await;
        assert!(json["success"].as_bool().unwrap());
        assert!(json["data"]["list"].is_array());
    }
}

#[tokio::test]
async fn test_reg_code_validation_errors() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let invalid_reg_code_body = json!({
        "code": "",
        "app_id": -1,
        "valid_days": -5,
        "max_devices": 0,
        "status": 99
    });
    let response = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&invalid_reg_code_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(response, "create_reg_code_response").await;
    assert_eq!(json["success"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_time_reg_code_activation_and_revalidate() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let app_key = helpers::unique_name("KEY");
    let create_app_body = json!({
        "name": helpers::unique_name("TimeApp"),
        "app_id": helpers::unique_name("com.test.time"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": app_key,
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "create_app_time_activation").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let code = helpers::unique_name("TIME_CODE");
    let create_rc = json!({
        "code": code,
        "app_id": app_id,
        "valid_days": 7,
        "max_devices": 1,
        "status": 0,
        "code_type": 0
    });
    let _ = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_rc)
        .send(&ctx.app)
        .await;

    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code":code, "app_key":app_key, "device_id":"dev-time-1"}))
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "validate_time_first").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["code_type"].as_i64().unwrap(), 0);
    assert!(json["data"]["expire_time"].is_string());

    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code":code, "app_key":app_key, "device_id":"dev-time-1"}))
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "validate_time_second").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["code_type"].as_i64().unwrap(), 0);
    assert!(json["data"]["expire_time"].is_string());
}

#[tokio::test]
async fn test_count_reg_code_activation_and_exhaust() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let app_key = helpers::unique_name("KEY");
    let create_app_body = json!({
        "name": helpers::unique_name("CountApp"),
        "app_id": helpers::unique_name("com.test.count"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": app_key,
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "create_app_count_activation").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    dotenvy::from_filename(".env.test").unwrap();
    let pool = sqlx::PgPool::connect(&helpers::db_url_from_env())
        .await
        .unwrap();
    sqlx::query("update apps set code_type = 1 where id = $1")
        .bind(app_id)
        .execute(&pool)
        .await
        .unwrap();

    let code = helpers::unique_name("COUNT_CODE");
    let create_rc = json!({
        "code": code,
        "app_id": app_id,
        "valid_days": 0,
        "max_devices": 1,
        "status": 0,
        "code_type": 1,
        "total_count": 1
    });
    let _ = TestClient::post(helpers::get_url("/api/admin/reg_codes"))
        .add_header("authorization", format!("Bearer {}", ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_rc)
        .send(&ctx.app)
        .await;

    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code":code, "app_key":app_key, "device_id":"dev-count-1"}))
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "validate_count_first").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["code_type"].as_i64().unwrap(), 1);
    assert_eq!(json["data"]["remaining_count"].as_i64().unwrap(), 0);

    let resp = TestClient::post(helpers::get_url("/api/reg/validate"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"code":code, "app_key":app_key, "device_id":"dev-count-1"}))
        .send(&ctx.app)
        .await;
    let json = print_response_body_get_json(resp, "validate_count_second").await;
    assert!(!json["success"].as_bool().unwrap());
}
