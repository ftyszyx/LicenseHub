use app_server::apis::payment_handler::{OrderStatus, caidou_sign};
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
use std::collections::BTreeMap;

mod helpers;

#[tokio::test]
async fn test_create_order_and_caidou_notify_delivers_reg_code() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let create_app_body = json!({
        "name": helpers::unique_name("PayApp"),
        "app_id": helpers::unique_name("com.pay.app"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": helpers::unique_name("PAYKEY"),
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_pay_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let plan_body = json!({
        "app_id": app_id,
        "name": "30 day license",
        "description": "test plan",
        "price_cents": 123,
        "code_type": 0,
        "valid_days": 30,
        "total_count": null,
        "status": 1,
        "sort_order": 0
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/plans"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&plan_body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_plan").await;
    let plan_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::post(helpers::get_url("/api/orders"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"plan_id": plan_id, "pay_type": "alipay"}))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_order").await;
    assert!(json["success"].as_bool().unwrap());
    let order_no = json["data"]["order_no"].as_str().unwrap().to_string();
    assert_eq!(
        json["data"]["status"].as_i64().unwrap(),
        OrderStatus::Pending as i64
    );

    let mut params = BTreeMap::new();
    params.insert("pid".to_string(), "1001".to_string());
    params.insert("trade_no".to_string(), helpers::unique_name("CD"));
    params.insert("out_trade_no".to_string(), order_no.clone());
    params.insert("type".to_string(), "alipay".to_string());
    params.insert("name".to_string(), "30 day license".to_string());
    params.insert("money".to_string(), "1.23".to_string());
    params.insert("trade_status".to_string(), "TRADE_SUCCESS".to_string());
    let sign = caidou_sign(&params, "test_key");

    let notify_body = json!({
        "pid": "1001",
        "trade_no": params["trade_no"],
        "out_trade_no": order_no,
        "type": "alipay",
        "name": "30 day license",
        "money": "1.23",
        "trade_status": "TRADE_SUCCESS",
        "sign": sign,
        "sign_type": "MD5"
    });
    let resp = TestClient::post(helpers::get_url("/api/pay/caidou/notify"))
        .add_header("content-type", "application/json", true)
        .json(&notify_body)
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));

    let resp = TestClient::get(helpers::get_url(&format!("/api/orders/{}", order_no)))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "get_delivered_order").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["status"].as_i64().unwrap(),
        OrderStatus::Delivered as i64
    );
    assert!(json["data"].get("pay_url").is_none());
    assert!(json["data"].get("qr_code").is_none());
    assert!(json["data"].get("url_scheme").is_none());
    assert!(
        json["data"]["reg_code"]
            .as_str()
            .unwrap()
            .starts_with("LH-")
    );

    let resp = TestClient::post(helpers::get_url("/api/pay/caidou/notify"))
        .add_header("content-type", "application/json", true)
        .json(&notify_body)
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));

    let resp = TestClient::get(helpers::get_url(
        "/api/admin/orders/list?page=1&page_size=10",
    ))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json =
        helpers::print_response_body_get_json(resp, "admin_order_list_after_duplicate_notify")
            .await;
    let matching: Vec<_> = json["data"]["list"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["order_no"].as_str() == Some(order_no.as_str()))
        .collect();
    assert_eq!(matching.len(), 1);
    assert!(matching[0]["reg_code"].as_str().unwrap().starts_with("LH-"));
}

#[tokio::test]
async fn test_public_products_list() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let resp = TestClient::get(helpers::get_url("/api/products"))
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "public_products").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"].is_array());
}
