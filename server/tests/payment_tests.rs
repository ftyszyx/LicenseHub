use app_server::apis::payment_handler::{OrderStatus, process_payment_notification};
use payment_adapter::{PaymentNotification, PaymentStatus};
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;

mod helpers;

#[tokio::test]
async fn test_create_order_and_payment_notification_delivers_reg_code() {
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
        .json(&json!({"plan_id": plan_id, "pay_type": "wechat_native"}))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_order").await;
    assert!(json["success"].as_bool().unwrap());
    let order_no = json["data"]["order_no"].as_str().unwrap().to_string();
    assert_eq!(
        json["data"]["status"].as_i64().unwrap(),
        OrderStatus::Pending as i64
    );
    assert_eq!(json["data"]["provider"].as_str().unwrap(), "wechat");

    let notification = PaymentNotification {
        provider: "wechat".to_string(),
        pay_type: "wechat_native".to_string(),
        out_trade_no: order_no.clone(),
        provider_trade_no: Some(helpers::unique_name("WX")),
        amount_cents: 123,
        status: PaymentStatus::Success,
        raw_payload: json!({"source": "unit-test"}),
    };
    process_payment_notification(&ctx.app_state, notification.clone())
        .await
        .unwrap();

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

    process_payment_notification(&ctx.app_state, notification)
        .await
        .unwrap();

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

#[tokio::test]
async fn test_admin_payment_channels_crud() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let pay_type = helpers::unique_name("alipay_page_test")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .take(32)
        .collect::<String>();
    let create_body = json!({
        "name": "Alipay test channel",
        "provider": "alipay",
        "pay_type": pay_type,
        "status": 1,
        "sort_order": 5,
        "config": {
            "app_id": "2021000000000000",
            "app_private_key_pem": "-----BEGIN PRIVATE KEY-----\\ntest\\n-----END PRIVATE KEY-----",
            "alipay_public_key_pem": "-----BEGIN PUBLIC KEY-----\\ntest\\n-----END PUBLIC KEY-----",
            "gateway_url": "",
            "seller_id": ""
        }
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/payment-channels"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_payment_channel").await;
    assert!(json["success"].as_bool().unwrap());
    let channel_id = json["data"]["id"].as_i64().unwrap() as i32;
    assert_eq!(json["data"]["provider"].as_str().unwrap(), "alipay");
    assert_eq!(json["data"]["pay_type"].as_str().unwrap(), pay_type);
    assert_eq!(
        json["data"]["config"]["gateway_url"].as_str().unwrap(),
        "https://openapi.alipay.com/gateway.do"
    );

    let resp = TestClient::get(helpers::get_url(
        "/api/admin/payment-channels/list?page=1&page_size=20&provider=alipay&status=1",
    ))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "list_payment_channels").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(
        json["data"]["list"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"].as_i64() == Some(channel_id as i64))
    );

    let resp = TestClient::put(helpers::get_url(&format!(
        "/api/admin/payment-channels/{}",
        channel_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({
        "name": "Alipay disabled channel",
        "status": 0,
        "sort_order": 9
    }))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "update_payment_channel").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["status"].as_i64().unwrap(), 0);
    assert_eq!(json["data"]["sort_order"].as_i64().unwrap(), 9);

    let resp = TestClient::delete(helpers::get_url(&format!(
        "/api/admin/payment-channels/{}",
        channel_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "delete_payment_channel").await;
    assert!(json["success"].as_bool().unwrap());

    let wechat_pay_type = helpers::unique_name("wechat_native_test")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .take(32)
        .collect::<String>();
    let resp = TestClient::post(helpers::get_url("/api/admin/payment-channels"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "name": "WeChat test channel",
            "provider": "wechat",
            "pay_type": wechat_pay_type,
            "status": 1,
            "sort_order": 1,
            "config": {
                "app_id": "wx1234567890abcdef",
                "mch_id": "1900000001",
                "merchant_serial_no": "ABCDEF1234567890",
                "merchant_private_key_pem": "-----BEGIN PRIVATE KEY-----\\ntest\\n-----END PRIVATE KEY-----",
                "api_v3_key": "12345678901234567890123456789012",
                "wechatpay_public_key_id": "PUB_KEY_ID_0114236900992025041500086300000000",
                "wechatpay_public_key_pem": "-----BEGIN PUBLIC KEY-----\\ntest\\n-----END PUBLIC KEY-----",
                "api_base_url": ""
            }
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_wechat_payment_channel").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["provider"].as_str().unwrap(), "wechat");
    assert_eq!(
        json["data"]["config"]["api_base_url"].as_str().unwrap(),
        "https://api.mch.weixin.qq.com"
    );
    assert_eq!(
        json["data"]["config"]["wechatpay_public_key_id"]
            .as_str()
            .unwrap(),
        "PUB_KEY_ID_0114236900992025041500086300000000"
    );
    let legacy_wechat_pay_type = helpers::unique_name("wechat_legacy_test")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .take(32)
        .collect::<String>();
    sqlx::query(
        r#"
        INSERT INTO payment_channels (name, provider, pay_type, status, sort_order, config)
        VALUES ($1, 'wechat', $2, 1, 2, $3::jsonb)
        "#,
    )
    .bind("Legacy WeChat channel")
    .bind(&legacy_wechat_pay_type)
    .bind(
        json!({
            "app_id": "wxlegacy",
            "mch_id": "1900000002",
            "merchant_serial_no": "LEGACY123456",
            "merchant_private_key_pem": "-----BEGIN PRIVATE KEY-----\\nlegacy\\n-----END PRIVATE KEY-----",
            "api_v3_key": "12345678901234567890123456789012",
            "wechatpay_public_key_id": "PUB_KEY_ID_0114236900992025041500086300000002",
            "wechatpay_public_key_pem": "-----BEGIN PUBLIC KEY-----\\nlegacy\\n-----END PUBLIC KEY-----",
            "api_base_url": ""
        })
        .to_string(),
    )
    .execute(
        &sqlx::PgPool::connect(&ctx.app_state.config.database.db_url)
            .await
            .unwrap(),
    )
    .await
    .unwrap();

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/admin/payment-channels/list?page=1&page_size=20&pay_type={}",
        legacy_wechat_pay_type
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json =
        helpers::print_response_body_get_json(resp, "list_legacy_wechat_payment_channel").await;
    assert!(json["success"].as_bool().unwrap());
    let legacy = &json["data"]["list"].as_array().unwrap()[0];
    assert_eq!(
        legacy["config"]["wechatpay_public_key_pem"]
            .as_str()
            .unwrap(),
        "-----BEGIN PUBLIC KEY-----\\nlegacy\\n-----END PUBLIC KEY-----"
    );
    assert_eq!(
        legacy["config"]["wechatpay_public_key_id"]
            .as_str()
            .unwrap(),
        "PUB_KEY_ID_0114236900992025041500086300000002"
    );
}
