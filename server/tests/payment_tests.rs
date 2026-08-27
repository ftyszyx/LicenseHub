use app_server::apis::distribution_handler::{SettlementPaymentProof, mark_settlement_paid_impl};
use app_server::apis::payment_handler::{
    ConfirmOrderRefundReq, OrderStatus, confirm_order_refund_impl, process_payment_notification,
};
use app_server::core::my_error::AppError;
use payment_adapter::{PaymentNotification, PaymentStatus};
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
use std::time::Duration;

mod helpers;

async fn enable_distribution(pool: &sqlx::PgPool, holding_days: i32, min_withdraw_cents: i32) {
    for (key, value) in [
        ("distribution_enabled", "true".to_string()),
        ("distribution_holding_days", holding_days.to_string()),
        (
            "distribution_min_withdraw_cents",
            min_withdraw_cents.to_string(),
        ),
    ] {
        sqlx::query("update system_settings set value = $2, updated_at = now() where key = $1")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await
            .unwrap();
    }
}

fn payment_proof_multipart(
    boundary: &str,
    payment_reference: &str,
    file_name: &str,
    content_type: &str,
    content: &[u8],
) -> Vec<u8> {
    let mut body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"payment_reference\"\r\n\r\n{payment_reference}\r\n--{boundary}\r\nContent-Disposition: form-data; name=\"proof\"; filename=\"{file_name}\"\r\nContent-Type: {content_type}\r\n\r\n"
    )
    .into_bytes();
    body.extend_from_slice(content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    body
}

async fn create_delivered_referral_order(
    ctx: &helpers::TestContext,
    pool: &sqlx::PgPool,
    price_cents: i32,
    label: &str,
) -> (i32, String) {
    let referral_code: String = sqlx::query_scalar("select referral_code from users where id = 1")
        .fetch_one(pool)
        .await
        .unwrap();
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "name": helpers::unique_name(&format!("{label}App")),
            "app_id": helpers::unique_name(&format!("com.{label}.app")),
            "app_vername": "1.0.0",
            "app_vercode": 1,
            "app_download_url": "https://example.com/dl",
            "app_res_url": "https://example.com/res",
            "app_update_info": "",
            "app_valid_key": helpers::unique_name(&format!("{label}KEY")),
            "trial_days": 0,
            "sort_order": 0,
            "status": 1
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_settlement_test_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::post(helpers::get_url("/api/admin/plans"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "app_id": app_id,
            "name": format!("{label} plan"),
            "description": null,
            "price_cents": price_cents,
            "code_type": 0,
            "valid_days": 30,
            "total_count": null,
            "status": 1,
            "sort_order": 0
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_settlement_test_plan").await;
    let plan_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::post(helpers::get_url("/api/orders"))
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "plan_id": plan_id,
            "pay_type": "wechat_native",
            "referral_code": referral_code,
            "buyer_email": format!("{}@example.com", label.to_ascii_lowercase())
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_settlement_test_order").await;
    let order_id = json["data"]["id"].as_i64().unwrap() as i32;
    let order_no = json["data"]["order_no"].as_str().unwrap().to_string();
    process_payment_notification(
        &ctx.app_state,
        PaymentNotification {
            provider: "wechat".to_string(),
            pay_type: "wechat_native".to_string(),
            out_trade_no: order_no.clone(),
            provider_trade_no: Some(helpers::unique_name("WX-SETTLEMENT")),
            amount_cents: price_cents,
            status: PaymentStatus::Success,
            raw_payload: json!({"source": "settlement-test"}),
        },
    )
    .await
    .unwrap();
    (order_id, order_no)
}

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
        .json(&json!({
            "plan_id": plan_id,
            "pay_type": "wechat_native",
            "buyer_email": "payment-test@example.com"
        }))
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
async fn test_confirm_order_refund_revokes_entitlement_and_cancels_commission() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    helpers::seed_test_license_signing_key(&ctx).await;

    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    sqlx::query(
        "update system_settings set value = 'true', updated_at = now() where key = 'distribution_enabled'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let referral_code: String = sqlx::query_scalar("select referral_code from users where id = 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    let app_key = helpers::unique_name("REFUNDKEY");
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "name": helpers::unique_name("RefundApp"),
            "app_id": helpers::unique_name("com.refund.app"),
            "app_vername": "1.0.0",
            "app_vercode": 1,
            "app_download_url": "https://example.com/dl",
            "app_res_url": "https://example.com/res",
            "app_update_info": "",
            "app_valid_key": app_key,
            "trial_days": 0,
            "max_devices": 2,
            "sort_order": 0,
            "status": 1
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_refund_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::post(helpers::get_url("/api/admin/plans"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "app_id": app_id,
            "name": "refund plan",
            "description": null,
            "price_cents": 5000,
            "code_type": 0,
            "valid_days": 30,
            "total_count": null,
            "status": 1,
            "sort_order": 0
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_refund_plan").await;
    let plan_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::post(helpers::get_url("/api/orders"))
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "plan_id": plan_id,
            "pay_type": "wechat_native",
            "referral_code": referral_code,
            "buyer_email": "refund-test@example.com"
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_refund_order").await;
    let order_id = json["data"]["id"].as_i64().unwrap() as i32;
    let order_no = json["data"]["order_no"].as_str().unwrap().to_string();

    let notification = PaymentNotification {
        provider: "wechat".to_string(),
        pay_type: "wechat_native".to_string(),
        out_trade_no: order_no.clone(),
        provider_trade_no: Some(helpers::unique_name("WX-REFUND")),
        amount_cents: 5000,
        status: PaymentStatus::Success,
        raw_payload: json!({"source": "refund-test"}),
    };
    process_payment_notification(&ctx.app_state, notification.clone())
        .await
        .unwrap();

    let reg_code_id: i32 = sqlx::query_scalar("select reg_code_id from orders where id = $1")
        .bind(order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let reg_code: String = sqlx::query_scalar("select code from reg_codes where id = $1")
        .bind(reg_code_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let bound_devices = [
        helpers::unique_name("refund-device-1"),
        helpers::unique_name("refund-device-2"),
    ];
    for device_id in &bound_devices {
        let resp = TestClient::post(helpers::get_url("/api/reg/bind"))
            .add_header("content-type", "application/json", true)
            .json(&json!({
                "app_key": app_key,
                "reg_code": reg_code,
                "device_id": device_id
            }))
            .send(&ctx.app)
            .await;
        let json = helpers::print_response_body_get_json(resp, "bind_refund_device").await;
        assert!(json["success"].as_bool().unwrap());
    }

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/reg_codes/{reg_code_id}/revoke"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let revoke_json =
        helpers::print_response_body_get_json(resp, "revoke_paid_order_without_refund").await;
    assert!(revoke_json["success"].as_bool().unwrap());
    assert_eq!(revoke_json["data"]["status"].as_i64(), Some(4));

    let order_status_before_refund: i16 =
        sqlx::query_scalar("select status from orders where id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(order_status_before_refund, OrderStatus::Delivered as i16);
    let refund_count: i64 =
        sqlx::query_scalar("select count(*) from order_refunds where order_id = $1")
            .bind(order_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(refund_count, 0);

    // Recreate device-side authorization to prove that the refund path clears
    // every historical binding even after a prior standalone revocation.
    sqlx::query(
        "update app_devices set expire_time = now() + interval '30 days', remaining = 9 where app_id = $1 and device_id = any($2)",
    )
    .bind(app_id)
    .bind(&bound_devices)
    .execute(&pool)
    .await
    .unwrap();

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/orders/{order_id}/refund"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({
        "refund_reference": "WX-REFUND-TEST-001",
        "reason": "customer requested refund"
    }))
    .send(&ctx.app)
    .await;
    let refund_json = helpers::print_response_body_get_json(resp, "confirm_order_refund").await;
    assert!(refund_json["success"].as_bool().unwrap());
    assert_eq!(
        refund_json["data"]["status"].as_i64(),
        Some(OrderStatus::Refunded as i64)
    );
    assert_eq!(
        refund_json["data"]["refund"]["refund_reference"].as_str(),
        Some("WX-REFUND-TEST-001")
    );

    let refund_row: (i32, String, String, i16, i32) = sqlx::query_as(
        "select amount_cents, refund_reference, reason, status, operator_user_id from order_refunds where order_id = $1",
    )
    .bind(order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(refund_row.0, 5000);
    assert_eq!(refund_row.1, "WX-REFUND-TEST-001");
    assert_eq!(refund_row.2, "customer requested refund");
    assert_eq!(refund_row.3, 1);
    assert_eq!(refund_row.4, 1);

    let commission: (i16, Option<String>) = sqlx::query_as(
        "select status, cancel_reason from distribution_commissions where order_id = $1",
    )
    .bind(order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(commission.0, 4);
    assert_eq!(commission.1.as_deref(), Some("order_refunded"));

    let reg_code_status: i16 = sqlx::query_scalar("select status from reg_codes where id = $1")
        .bind(reg_code_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reg_code_status, 3);

    let revoked_devices: Vec<(Option<chrono::DateTime<chrono::FixedOffset>>, Option<i32>)> =
        sqlx::query_as(
            "select expire_time, remaining from app_devices where app_id = $1 and device_id = any($2)",
        )
        .bind(app_id)
        .bind(&bound_devices)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(revoked_devices.len(), 2);
    for (expire_time, remaining) in revoked_devices {
        assert!(expire_time.unwrap().timestamp() <= chrono::Utc::now().timestamp());
        assert_eq!(remaining, Some(0));
    }

    let refund_event_count: i64 = sqlx::query_scalar(
        "select count(*) from order_events where order_id = $1 and event_type = 'refund.confirmed'",
    )
    .bind(order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(refund_event_count, 1);

    let repeated = process_payment_notification(&ctx.app_state, notification)
        .await
        .unwrap();
    assert_eq!(repeated.status, OrderStatus::Refunded);
    assert_eq!(repeated.reg_code_id, Some(reg_code_id));

    let final_status: i16 = sqlx::query_scalar("select status from orders where id = $1")
        .bind(order_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(final_status, OrderStatus::Refunded as i16);
}

#[tokio::test]
async fn test_refund_rejects_pending_withdrawal_and_releases_locked_commission() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    enable_distribution(&pool, 0, 100).await;
    let (order_id, _) = create_delivered_referral_order(&ctx, &pool, 10_000, "LockedRefund").await;

    let resp = TestClient::get(helpers::get_url("/api/admin/me/distribution/summary"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    let summary = helpers::print_response_body_get_json(resp, "release_locked_commission").await;
    assert_eq!(
        summary["data"]["available_amount_cents"].as_i64(),
        Some(2000)
    );

    let resp = TestClient::post(helpers::get_url("/api/admin/me/distribution/settlements"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "amount_cents": 2000,
            "alipay_account": "locked@example.com",
            "real_name": "锁定测试"
        }))
        .send(&ctx.app)
        .await;
    let settlement = helpers::print_response_body_get_json(resp, "create_locked_settlement").await;
    let settlement_id = settlement["data"]["id"].as_i64().unwrap();
    assert_eq!(settlement["data"]["status"].as_i64(), Some(0));

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/orders/{order_id}/refund"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({
        "refund_reference": "LOCKED-REFUND-001",
        "reason": "refund while withdrawal pending"
    }))
    .send(&ctx.app)
    .await;
    let refund = helpers::print_response_body_get_json(resp, "refund_locked_commission").await;
    assert!(refund["success"].as_bool().unwrap());

    let settlement_row: (i16, Option<String>) =
        sqlx::query_as("select status, reject_reason from distribution_settlements where id = $1")
            .bind(settlement_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(settlement_row.0, 2);
    assert!(settlement_row.1.unwrap().contains("订单退款"));
    let commission: (i16, i32, i32) = sqlx::query_as(
        "select status, locked_amount_cents, cancelled_amount_cents from distribution_commissions where order_id = $1",
    )
    .bind(order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(commission, (4, 0, 2000));
}

#[tokio::test]
async fn test_refund_and_payment_confirmation_serialize_on_distribution_user() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    enable_distribution(&pool, 0, 100).await;
    let (order_id, _) =
        create_delivered_referral_order(&ctx, &pool, 10_000, "ConcurrentRefund").await;

    let resp = TestClient::get(helpers::get_url("/api/admin/me/distribution/summary"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    let summary =
        helpers::print_response_body_get_json(resp, "release_concurrent_commission").await;
    assert_eq!(
        summary["data"]["available_amount_cents"].as_i64(),
        Some(2000)
    );

    let resp = TestClient::post(helpers::get_url("/api/admin/me/distribution/settlements"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "amount_cents": 2000,
            "alipay_account": "concurrent@example.com",
            "real_name": "Concurrent Test"
        }))
        .send(&ctx.app)
        .await;
    let settlement =
        helpers::print_response_body_get_json(resp, "create_concurrent_settlement").await;
    let settlement_id = settlement["data"]["id"].as_i64().unwrap();

    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("select id from users where id = 1 for update")
        .fetch_one(&mut *blocker)
        .await
        .unwrap();

    let refund_state = ctx.app_state.clone();
    let refund_task = tokio::spawn(async move {
        confirm_order_refund_impl(
            &refund_state,
            1,
            order_id,
            ConfirmOrderRefundReq {
                refund_reference: "CONCURRENT-REFUND-001".to_string(),
                reason: "concurrent refund test".to_string(),
            },
        )
        .await
    });
    let paid_state = ctx.app_state.clone();
    let paid_task = tokio::spawn(async move {
        mark_settlement_paid_impl(
            &paid_state,
            1,
            settlement_id,
            SettlementPaymentProof {
                payment_reference: "CONCURRENT-PAYMENT-001".to_string(),
                file_name: "proof.png".to_string(),
                content_type: "image/png".to_string(),
                content: vec![137, 80, 78, 71],
            },
        )
        .await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(!refund_task.is_finished());
    assert!(!paid_task.is_finished());
    blocker.commit().await.unwrap();

    let (refund_result, paid_result) = tokio::time::timeout(Duration::from_secs(10), async {
        (refund_task.await.unwrap(), paid_task.await.unwrap())
    })
    .await
    .expect("distribution operations should finish without a database deadlock");
    assert!(refund_result.is_ok(), "refund failed: {refund_result:?}");
    match paid_result {
        Ok(settlement) => assert_eq!(settlement.status, 1),
        Err(AppError::BusinessLogic { code, .. }) => {
            assert_eq!(code, "SETTLEMENT_NOT_PENDING")
        }
        Err(error) => panic!("unexpected payment confirmation error: {error}"),
    }
}

#[tokio::test]
async fn test_settled_refund_creates_debt_and_future_commission_offsets_it() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;
    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    enable_distribution(&pool, 0, 100).await;
    let (first_order_id, _) =
        create_delivered_referral_order(&ctx, &pool, 50_000, "SettledRefund").await;

    let resp = TestClient::get(helpers::get_url("/api/admin/me/distribution/summary"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    let summary = helpers::print_response_body_get_json(resp, "release_settled_commission").await;
    assert_eq!(
        summary["data"]["available_amount_cents"].as_i64(),
        Some(10_000)
    );

    let resp = TestClient::post(helpers::get_url("/api/admin/me/distribution/settlements"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "amount_cents": 6000,
            "alipay_account": "paid@example.com",
            "real_name": "结算测试"
        }))
        .send(&ctx.app)
        .await;
    let settlement = helpers::print_response_body_get_json(resp, "create_paid_settlement").await;
    let settlement_id = settlement["data"]["id"].as_i64().unwrap();
    let boundary = "licensehub-payment-proof";
    let proof_content = [137, 80, 78, 71, 1, 2, 3];
    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/distribution/settlements/{settlement_id}/paid"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header(
        "content-type",
        format!("multipart/form-data; boundary={boundary}"),
        true,
    )
    .body(payment_proof_multipart(
        boundary,
        "ALIPAY-OFFLINE-001",
        "proof.png",
        "image/png",
        &proof_content,
    ))
    .send(&ctx.app)
    .await;
    let paid = helpers::print_response_body_get_json(resp, "mark_settlement_paid").await;
    assert!(paid["success"].as_bool().unwrap());
    assert_eq!(paid["data"]["status"].as_i64(), Some(1));
    assert_eq!(
        paid["data"]["payment_reference"].as_str(),
        Some("ALIPAY-OFFLINE-001")
    );

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/orders/{first_order_id}/refund"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({
        "refund_reference": "SETTLED-REFUND-001",
        "reason": "refund after offline payment"
    }))
    .send(&ctx.app)
    .await;
    let refund = helpers::print_response_body_get_json(resp, "refund_settled_commission").await;
    assert!(refund["success"].as_bool().unwrap());

    let first_commission: (i16, i32, i32) = sqlx::query_as(
        "select status, settled_amount_cents, cancelled_amount_cents from distribution_commissions where order_id = $1",
    )
    .bind(first_order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(first_commission, (3, 6000, 4000));
    let adjustment: (i32, i32, i16) = sqlx::query_as(
        "select amount_cents, offset_amount_cents, status from distribution_commission_adjustments where order_id = $1",
    )
    .bind(first_order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(adjustment, (-6000, 0, 0));

    let (second_order_id, _) =
        create_delivered_referral_order(&ctx, &pool, 50_000, "DebtOffset").await;
    let resp = TestClient::get(helpers::get_url("/api/admin/me/distribution/summary"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .send(&ctx.app)
        .await;
    let summary = helpers::print_response_body_get_json(resp, "offset_refund_debt").await;
    assert_eq!(summary["data"]["adjustment_debt_cents"].as_i64(), Some(0));
    assert_eq!(
        summary["data"]["available_amount_cents"].as_i64(),
        Some(4000)
    );

    let second_commission: (i16, i32) = sqlx::query_as(
        "select status, adjustment_amount_cents from distribution_commissions where order_id = $1",
    )
    .bind(second_order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(second_commission, (1, 6000));
    let adjustment: (i32, i16) = sqlx::query_as(
        "select offset_amount_cents, status from distribution_commission_adjustments where order_id = $1",
    )
    .bind(first_order_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(adjustment, (6000, 2));
    let proof_size: i32 = sqlx::query_scalar(
        "select octet_length(content) from distribution_settlement_proofs where settlement_id = $1",
    )
    .bind(settlement_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(proof_size, 7);
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
    assert_eq!(json["data"]["state"].as_str().unwrap(), "available");
    assert!(json["data"]["plans"].is_array());
}

#[tokio::test]
async fn test_public_products_follow_configured_sort_order() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let create_app_body = json!({
        "name": helpers::unique_name("SortedPayApp"),
        "app_id": helpers::unique_name("com.sorted.pay.app"),
        "website_url": "https://example.com/sorted-app",
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": helpers::unique_name("SORTKEY"),
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
    let json = helpers::print_response_body_get_json(resp, "create_sorted_pay_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let mut plan_ids = Vec::new();
    for (name, sort_order) in [("last", 20), ("first", 10), ("second", 10)] {
        let resp = TestClient::post(helpers::get_url("/api/admin/plans"))
            .add_header("authorization", helpers::bearer(&ctx.token), true)
            .add_header("content-type", "application/json", true)
            .json(&json!({
                "app_id": app_id,
                "name": name,
                "description": null,
                "price_cents": 100,
                "code_type": 0,
                "valid_days": 30,
                "total_count": null,
                "status": 1,
                "sort_order": sort_order
            }))
            .send(&ctx.app)
            .await;
        let json = helpers::print_response_body_get_json(resp, "create_sorted_plan").await;
        plan_ids.push(json["data"]["id"].as_i64().unwrap());
    }

    let resp = TestClient::get(helpers::get_url(&format!("/api/products?app_id={app_id}")))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "sorted_public_products").await;
    assert_eq!(
        json["data"]["app_website_url"].as_str().unwrap(),
        "https://example.com/sorted-app"
    );
    assert_eq!(
        json["data"]["plans"][0]["app_website_url"]
            .as_str()
            .unwrap(),
        "https://example.com/sorted-app"
    );
    let ids: Vec<i64> = json["data"]["plans"]
        .as_array()
        .unwrap()
        .iter()
        .map(|plan| plan["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![plan_ids[1], plan_ids[2], plan_ids[0]]);

    let resp = TestClient::get(helpers::get_url("/api/products"))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "grouped_public_products").await;
    let grouped_plan = json["data"]["plans"]
        .as_array()
        .unwrap()
        .iter()
        .find(|plan| plan["id"].as_i64() == Some(plan_ids[0]))
        .unwrap();
    assert_eq!(
        grouped_plan["app_website_url"].as_str().unwrap(),
        "https://example.com/sorted-app"
    );

    let resp = TestClient::put(helpers::get_url(&format!(
        "/api/admin/plans/{}",
        plan_ids[0]
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({ "sort_order": 5 }))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "update_plan_sort_order").await;
    assert_eq!(json["data"]["sort_order"].as_i64().unwrap(), 5);

    let resp = TestClient::get(helpers::get_url(&format!("/api/products?app_id={app_id}")))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "resorted_public_products").await;
    assert_eq!(
        json["data"]["plans"][0]["id"].as_i64().unwrap(),
        plan_ids[0]
    );
}

#[tokio::test]
async fn test_public_products_hide_disabled_app_and_reject_order() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let create_app_body = json!({
        "name": helpers::unique_name("DisabledPayApp"),
        "app_id": helpers::unique_name("com.disabled.pay.app"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/dl",
        "app_res_url": "https://example.com/res",
        "app_update_info": "",
        "app_valid_key": helpers::unique_name("PAYKEY"),
        "trial_days": 0,
        "sort_order": 0,
        "status": 0
    });
    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&create_app_body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_disabled_pay_app").await;
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let plan_body = json!({
        "app_id": app_id,
        "name": "disabled app license",
        "description": "test disabled app plan",
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
    let json = helpers::print_response_body_get_json(resp, "create_disabled_app_plan").await;
    let plan_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/products?app_id={}",
        app_id
    )))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "disabled_app_public_products").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["state"].as_str().unwrap(), "app_disabled");
    assert_eq!(json["data"]["app_id"].as_i64().unwrap(), app_id as i64);
    assert_eq!(json["data"]["plans"].as_array().unwrap().len(), 0);

    let resp = TestClient::get(helpers::get_url("/api/products"))
        .send(&ctx.app)
        .await;
    let json =
        helpers::print_response_body_get_json(resp, "all_public_products_without_disabled_app")
            .await;
    assert!(json["success"].as_bool().unwrap());
    assert!(
        json["data"]["plans"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["id"].as_i64() != Some(plan_id as i64))
    );

    let resp = TestClient::post(helpers::get_url("/api/orders"))
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "plan_id": plan_id,
            "pay_type": "wechat_native",
            "buyer_email": "disabled-plan@example.com"
        }))
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_disabled_app_order").await;
    assert!(!json["success"].as_bool().unwrap());
    assert!(json["message"].as_str().unwrap().contains("APP_DISABLED"));
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
