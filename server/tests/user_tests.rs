use app_server::core::constants;
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
mod helpers;

#[tokio::test]
async fn test_get_users_list() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let response = TestClient::get(helpers::get_url(
        "/api/admin/users/list?page=1&page_size=10",
    ))
    .add_header("authorization", format!("Bearer {}", token), true)
    .send(&app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "get_users_list").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["list"].is_array());
    assert!(json["data"]["total"].is_number());
    assert!(json["data"]["page"].is_number());
    assert!(
        json["data"]["list"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["user"].get("password").is_none())
    );
}

#[tokio::test]
async fn test_get_users_list_default_pagination() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let response = TestClient::get(helpers::get_url("/api/admin/users/list"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
}

#[tokio::test]
async fn test_get_users_list_with_search() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let response = TestClient::get(helpers::get_url(
        "/api/admin/users/list?page=1&page_size=10&username=test",
    ))
    .add_header("authorization", format!("Bearer {}", token), true)
    .send(&app)
    .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "get_users_list_with_search").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["list"].is_array());
}

#[tokio::test]
async fn test_create_user() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let create_user_body = json!({
        "username": format!("new_user_{}", chrono::Utc::now().timestamp()),
        "password": "newuserpass123"
    });
    let response = TestClient::post(helpers::get_url("/api/admin/users"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_user_body)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "create_user").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"]["user"]["id"].is_number());
    assert!(json["data"]["user"]["username"].is_string());
    assert!(json["data"]["user"].get("password").is_none());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let create_user_body = json!({
        "username": format!("get_user_{}", chrono::Utc::now().timestamp()),
        "password": "getuserpass123",
        "role_id": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/users"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_user_body)
        .send(&app)
        .await;
    let json = helpers::print_response_body_get_json(response, "create_user_for_get_by_id").await;
    let user_id = json["data"]["user"]["id"].as_i64().unwrap();
    let response = TestClient::get(helpers::get_url(&format!("/api/admin/users/{}", user_id)))
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "get_user_by_id").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["user"]["id"].as_i64().unwrap(), user_id);
    assert!(json["data"]["user"].get("password").is_none());
}

#[tokio::test]
async fn test_update_user() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let create_user_body = json!({
        "username": format!("update_user_{}", chrono::Utc::now().timestamp()),
        "password": "updateuserpass123",
        "role_id": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/users"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_user_body)
        .send(&app)
        .await;
    let json = helpers::print_response_body_get_json(response, "create_user_for_update").await;
    let user_id = json["data"]["user"]["id"].as_i64().unwrap();
    let update_user_body = json!({
        "username": format!("updated_user_{}", chrono::Utc::now().timestamp()),
        "balance": 1000
    });
    let response = TestClient::put(helpers::get_url(&format!("/api/admin/users/{}", user_id)))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&update_user_body)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "update_user").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["user"]["id"].as_i64().unwrap(), user_id);
}

#[tokio::test]
async fn test_delete_user() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let create_user_body = json!({
        "username": format!("delete_user_{}", chrono::Utc::now().timestamp()),
        "password": "deleteuserpass123",
        "role_id": 1
    });
    let response = TestClient::post(helpers::get_url("/api/admin/users"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_user_body)
        .send(&app)
        .await;
    let json = helpers::print_response_body_get_json(response, "create_user_for_delete").await;
    let user_id = json["data"]["user"]["id"].as_i64().unwrap();
    let response = TestClient::delete(helpers::get_url(&format!("/api/admin/users/{}", user_id)))
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(response, "delete_user").await;
    assert!(json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let app = helpers::create_test_app().await;
    let token = helpers::login_admin(&app).await;
    let response = TestClient::get(helpers::get_url("/api/admin/users/99999"))
        .add_header("authorization", format!("Bearer {}", token), true)
        .send(&app)
        .await;
    let bodyjson = helpers::print_response_body_get_json(response, "get_nonexistent_user").await;
    assert_eq!(bodyjson["code"], constants::APP_NOT_FOUND);
}

#[tokio::test]
async fn test_users_without_auth() {
    let app = helpers::create_test_app().await;
    let response = TestClient::get(helpers::get_url("/api/admin/users/list"))
        .send(&app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::UNAUTHORIZED));
}
