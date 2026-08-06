mod helpers;

use app_server::apis::email_verification_handler::token_hash;
use helpers::*;
use salvo::http::StatusCode;
use salvo::test::{ResponseExt, TestClient};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn email_verification_send_requires_only_email_and_enforces_cooldown() {
    let _guard = db_lock().await;
    let ctx = TestContext::new().await;
    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    sqlx::query("UPDATE system_settings SET value = 'true' WHERE key = 'registration_enabled'")
        .execute(&pool)
        .await
        .unwrap();

    let email = format!("{}@example.com", unique_name("cooldown"));
    let payload = json!({ "email": email });
    let mut first_response = TestClient::post(get_url("/api/auth/email-verifications"))
        .add_header("content-type", "application/json", true)
        .json(&payload)
        .send(&ctx.app)
        .await;
    assert_eq!(first_response.status_code, Some(StatusCode::OK));
    let first_json = first_response
        .take_json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(first_json["success"], true);
    assert_eq!(first_json["data"]["resend_after_seconds"], 60);

    let mut second_response = TestClient::post(get_url("/api/auth/email-verifications"))
        .add_header("content-type", "application/json", true)
        .json(&payload)
        .send(&ctx.app)
        .await;
    assert_eq!(second_response.status_code, Some(StatusCode::OK));
    let second_json = second_response
        .take_json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(second_json["success"], false);
    assert!(
        second_json["message"]
            .as_str()
            .unwrap()
            .contains("发送过于频繁")
    );

    let challenge_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM email_verification_challenges WHERE email = $1")
            .bind(&email)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(challenge_count, 1);
}

#[tokio::test]
async fn verified_registration_claims_guest_order_and_exposes_it_to_the_user() {
    let _guard = db_lock().await;
    let ctx = TestContext::new().await;
    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    sqlx::query("UPDATE system_settings SET value = 'true' WHERE key = 'registration_enabled'")
        .execute(&pool)
        .await
        .unwrap();

    let email = "buyer@example.com";
    let challenge_id = Uuid::new_v4();
    let verification_token = "registration-test-token";
    sqlx::query(
        r#"INSERT INTO email_verification_challenges
           (id, email, purpose, code_hash, attempts, expires_at, resend_after, sent_at, verified_at, created_at)
           VALUES ($1, $2, 'register', 'test-code-hash', 0, NOW() + INTERVAL '10 minutes', NOW(), NOW(), NOW(), NOW())"#,
    )
    .bind(challenge_id)
    .bind(email)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO email_verification_tokens
           (token_hash, challenge_id, email, purpose, expires_at, created_at)
           VALUES ($1, $2, $3, 'register', NOW() + INTERVAL '15 minutes', NOW())"#,
    )
    .bind(token_hash(verification_token))
    .bind(challenge_id)
    .bind(email)
    .execute(&pool)
    .await
    .unwrap();

    let app_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO apps
           (name, app_id, app_vername, app_vercode, app_download_url, app_res_url,
            manifest_extra, code_type, app_valid_key, trial_days, trial_num, sort_order, status)
           VALUES ('Test App', 'registration-order-test', '1.0', 1, '', '', '{}'::jsonb, 0, '', 0, 0, 0, 1)
           RETURNING id"#,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let plan_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO license_plans
           (app_id, name, price_cents, code_type, valid_days, status)
           VALUES ($1, 'Test Plan', 9900, 0, 30, 1)
           RETURNING id"#,
    )
    .bind(app_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let reg_code_id: i32 = sqlx::query_scalar(
        r#"INSERT INTO reg_codes
           (code, app_id, valid_days, max_devices, status, code_type)
           VALUES ('REG-CLAIM-TEST', $1, 30, 1, 1, 0)
           RETURNING id"#,
    )
    .bind(app_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO orders
           (order_no, plan_id, app_id, amount_cents, pay_type, status, provider,
            reg_code_id, buyer_email, paid_at)
           VALUES ('ORDER-CLAIM-TEST', $1, $2, 9900, 'alipay', 2, 'alipay', $3, $4, NOW())"#,
    )
    .bind(plan_id)
    .bind(app_id)
    .bind(reg_code_id)
    .bind(email)
    .execute(&pool)
    .await
    .unwrap();

    let mut register_response = TestClient::post(get_url("/api/register"))
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "username": "verified_buyer",
            "email": email,
            "password": "password123",
            "verification_token": verification_token
        }))
        .send(&ctx.app)
        .await;
    assert_eq!(register_response.status_code, Some(StatusCode::OK));
    let register_json = register_response
        .take_json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(register_json["success"], true);
    let token = register_json["data"]["token"].as_str().unwrap();

    let claimed_user_id: Option<i32> =
        sqlx::query_scalar("SELECT buyer_user_id FROM orders WHERE order_no = 'ORDER-CLAIM-TEST'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(claimed_user_id.is_some());
    let consumed: bool = sqlx::query_scalar(
        "SELECT consumed_at IS NOT NULL FROM email_verification_tokens WHERE token_hash = $1",
    )
    .bind(token_hash(verification_token))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(consumed);

    let mut orders_response = TestClient::get(get_url("/api/admin/me/orders?page=1&page_size=20"))
        .add_header("Authorization", bearer(token), true)
        .send(&ctx.app)
        .await;
    let orders_json = orders_response
        .take_json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(orders_json["success"], true);
    assert_eq!(orders_json["data"]["total"], 1);
    assert_eq!(
        orders_json["data"]["list"][0]["order_no"],
        "ORDER-CLAIM-TEST"
    );
    assert_eq!(orders_json["data"]["list"][0]["reg_code"], "REG-CLAIM-TEST");
}
