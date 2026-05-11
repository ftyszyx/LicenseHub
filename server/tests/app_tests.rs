use crate::helpers::print_response_body_get_json;
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
mod helpers;

#[tokio::test]
async fn test_get_apps_list() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let url = helpers::get_url("/api/admin/apps/list?page=1&page_size=10");
    let response = TestClient::get(url)
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "get_apps_list").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["list"].is_array());
    assert!(json["data"]["total"].is_number());
}

#[tokio::test]
async fn test_create_app() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;

    let create_app_body = json!({
        "name": format!("TestApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.test.app_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "app_update_info": "Test app for automated testing",
        "sort_order": 1,
        "status": 1
    });

    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&app)
        .await;
    let bodyjson = print_response_body_get_json(response, "create_app").await;
    assert!(bodyjson["success"].as_bool().unwrap());
    assert!(bodyjson["data"]["id"].is_number());
    assert!(bodyjson["data"]["name"].is_string());
}

#[tokio::test]
async fn test_get_app_by_id() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;

    // 先创建一个应用
    let create_app_body = json!({
        "name": format!("GetApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.get.app_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "app_update_info": "Test app",
        "sort_order": 1,
        "status": 1
    });

    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&app)
        .await;
    let json = print_response_body_get_json(response, "create_app_for_get_by_id").await;
    let app_id = json["data"]["id"].as_i64().unwrap();

    let url = helpers::get_url(&format!("/api/admin/apps/{}", app_id));
    // 然后通过 ID 获取应用
    let response = TestClient::get(url)
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "get_app_by_id").await;

    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["id"].as_i64().unwrap(), app_id);
}

#[tokio::test]
async fn test_update_app() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;

    // 先创建一个应用
    let create_app_body = json!({
        "name": format!("UpdateApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.update.app_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "app_update_info": "Test app",
        "sort_order": 1,
        "status": 1
    });

    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&app)
        .await;
    let bodyjson = print_response_body_get_json(response, "create_app").await;
    let app_id = bodyjson["data"]["id"].as_i64().unwrap();

    // 然后更新应用
    let update_app_body = json!({
        "name": format!("UpdatedApp_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.1",
        "status": 0
    });

    let url = helpers::get_url(&format!("/api/admin/apps/{}", app_id));
    let response = TestClient::put(url)
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&update_app_body)
        .send(&app)
        .await;
    let bodyjson = helpers::print_response_body_get_json(response, "update_app").await;
    assert!(bodyjson["success"].as_bool().unwrap());
    assert_eq!(bodyjson["data"]["id"].as_i64().unwrap(), app_id);
}

#[tokio::test]
async fn test_delete_app() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;

    // 先创建一个应用
    let create_app_body = json!({
        "name": format!("DeleteApp_{}", chrono::Utc::now().timestamp()),
        "app_id": format!("com.delete.app_{}", chrono::Utc::now().timestamp()),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download",
        "app_res_url": "https://example.com/resources",
        "app_update_info": "Test app",
        "sort_order": 1,
        "status": 1
    });

    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&app)
        .await;
    let json = print_response_body_get_json(response, "create_then_delete_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap();

    // 然后删除应用
    let url = helpers::get_url(&format!("/api/admin/apps/{}", app_id));
    let response = TestClient::delete(url)
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "delete_app").await;

    assert!(json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_apps_pagination() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;

    // 测试不同的分页参数
    let test_cases = vec![
        helpers::get_url("/api/admin/apps/list?page=1&page_size=5"),
        helpers::get_url("/api/admin/apps/list?page=2&page_size=10"),
        helpers::get_url("/api/admin/apps/list?page=1"), // 使用默认 page_size
        helpers::get_url("/api/admin/apps/list?page_size=20"), // 使用默认 page
        helpers::get_url("/api/admin/apps/list"),        // 使用所有默认值
    ];

    for uri in test_cases {
        let response = TestClient::get(uri)
            .add_header("authorization", format!("Bearer {}", token), true)
            .send(&app)
            .await;
        let bodyjson = print_response_body_get_json(response, "apps_pagination").await;
        assert!(bodyjson["success"].as_bool().unwrap());

        assert!(bodyjson["data"]["list"].is_array());
        assert!(bodyjson["data"]["total"].is_number());
        assert!(bodyjson["data"]["page"].is_number());
    }
}
