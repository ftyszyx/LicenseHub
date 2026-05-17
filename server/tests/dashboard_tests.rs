use app_server::apis::payment_handler::{OrderStatus, process_payment_notification};
use payment_adapter::{PaymentNotification, PaymentStatus};
use salvo::http::StatusCode;
use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn test_dashboard_stats_use_real_data() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let before = fetch_dashboard(&ctx).await;

    let create_app_body = json!({
        "name": helpers::unique_name("DashboardApp"),
        "app_id": helpers::unique_name("com.dashboard.app"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": helpers::unique_name("DASHKEY"),
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
    let json = helpers::print_response_body_get_json(resp, "dashboard_create_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let plan_body = json!({
        "app_id": app_id,
        "name": "Dashboard plan",
        "description": "dashboard test plan",
        "price_cents": 2599,
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
    let json = helpers::print_response_body_get_json(resp, "dashboard_create_plan").await;
    let plan_id = json["data"]["id"].as_i64().unwrap() as i32;

    let first_order_no = create_order(&ctx, plan_id).await;
    let _pending_order_no = create_order(&ctx, plan_id).await;

    process_payment_notification(
        &ctx.app_state,
        PaymentNotification {
            provider: "wechat".to_string(),
            pay_type: "wechat_native".to_string(),
            out_trade_no: first_order_no.clone(),
            provider_trade_no: Some(helpers::unique_name("WX")),
            amount_cents: 2599,
            status: PaymentStatus::Success,
            raw_payload: json!({"source": "dashboard-test"}),
        },
    )
    .await
    .unwrap();

    let after = fetch_dashboard(&ctx).await;
    assert_eq!(
        after["total_orders"].as_u64().unwrap() - before["total_orders"].as_u64().unwrap(),
        2
    );
    assert_eq!(
        after["new_orders_today"].as_u64().unwrap() - before["new_orders_today"].as_u64().unwrap(),
        2
    );
    assert_eq!(
        after["delivered_orders"].as_u64().unwrap() - before["delivered_orders"].as_u64().unwrap(),
        1
    );
    assert_eq!(
        after["pending_orders"].as_u64().unwrap() - before["pending_orders"].as_u64().unwrap(),
        1
    );
    assert_eq!(
        after["total_revenue_cents"].as_i64().unwrap()
            - before["total_revenue_cents"].as_i64().unwrap(),
        2599
    );
    assert_eq!(
        after["active_products"].as_u64().unwrap() - before["active_products"].as_u64().unwrap(),
        1
    );
    assert!(
        after["recent_orders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|order| {
                order["order_no"].as_str() == Some(first_order_no.as_str())
                    && order["status"].as_i64() == Some(OrderStatus::Delivered as i64)
            })
    );
}

async fn fetch_dashboard(ctx: &helpers::TestContext) -> serde_json::Value {
    let resp = TestClient::get(helpers::get_url("/api/admin/dashboard"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    assert_eq!(resp.status_code, Some(StatusCode::OK));
    let json = helpers::print_response_body_get_json(resp, "dashboard_stats").await;
    assert!(json["success"].as_bool().unwrap());
    json["data"].clone()
}

async fn create_order(ctx: &helpers::TestContext, plan_id: i32) -> String {
    let resp = TestClient::post(helpers::get_url("/api/orders"))
        .add_header("content-type", "application/json", true)
        .json(&json!({"plan_id": plan_id, "pay_type": "wechat_native"}))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "dashboard_create_order").await;
    assert!(json["success"].as_bool().unwrap());
    json["data"]["order_no"].as_str().unwrap().to_string()
}
