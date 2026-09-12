use crate::helpers::print_response_body_get_json;
use salvo::test::TestClient;
use serde_json::json;
mod helpers;

#[tokio::test]
async fn test_devices_list_includes_bound_reg_codes_and_sorts_by_count() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let app_key = helpers::unique_name("DEVICE_LIST_KEY");
    let response = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "name": helpers::unique_name("DeviceListApp"),
            "app_id": helpers::unique_name("com.device.list"),
            "app_vername": "1.0.0",
            "app_vercode": 1,
            "app_download_url": "https://example.com/dl",
            "app_res_url": "https://example.com/res",
            "app_update_info": "",
            "app_valid_key": app_key,
            "code_type": 0,
            "trial_days": 0,
            "trial_num": 0,
            "max_devices": 2,
            "sort_order": 0,
            "status": 1
        }))
        .send(&ctx.app)
        .await;
    let body = print_response_body_get_json(response, "create_app_for_device_list").await;
    assert!(body["success"].as_bool().unwrap());
    let app_id = body["data"]["id"].as_i64().unwrap() as i32;

    let pool = sqlx::PgPool::connect(&ctx.get_db_url()).await.unwrap();
    let device_names = [
        helpers::unique_name("device-with-two-codes"),
        helpers::unique_name("device-with-one-code"),
        helpers::unique_name("device-without-codes"),
    ];
    let mut device_ids = Vec::new();
    for device_name in &device_names {
        let id: i32 = sqlx::query_scalar(
            "insert into app_devices (app_id, device_id) values ($1, $2) returning id",
        )
        .bind(app_id)
        .bind(device_name)
        .fetch_one(&pool)
        .await
        .unwrap();
        device_ids.push(id);
    }

    let code_names = [
        helpers::unique_name("bound-code"),
        helpers::unique_name("revoked-code"),
        helpers::unique_name("other-device-code"),
    ];
    let statuses = [2_i16, 4_i16, 2_i16];
    let mut reg_code_ids = Vec::new();
    for (code, status) in code_names.iter().zip(statuses) {
        let id: i32 = sqlx::query_scalar(
            r#"
            insert into reg_codes (code, app_id, valid_days, max_devices, status, code_type)
            values ($1, $2, 10, 2, $3, 0)
            returning id
            "#,
        )
        .bind(code)
        .bind(app_id)
        .bind(status)
        .fetch_one(&pool)
        .await
        .unwrap();
        reg_code_ids.push(id);
    }

    sqlx::query(
        r#"
        insert into reg_code_devices (reg_code_id, device_id, created_at)
        values
            ($1, $3, now() - interval '2 hours'),
            ($2, $3, now() - interval '1 hour'),
            ($4, $5, now())
        "#,
    )
    .bind(reg_code_ids[0])
    .bind(reg_code_ids[1])
    .bind(device_ids[0])
    .bind(reg_code_ids[2])
    .bind(device_ids[1])
    .execute(&pool)
    .await
    .unwrap();

    let response = TestClient::get(helpers::get_url(&format!(
        "/api/admin/devices/list?app_id={app_id}&sort_by=bound_reg_code_count&sort_order=desc&page=1&page_size=10"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let body = print_response_body_get_json(response, "devices_by_code_count_desc").await;
    assert!(body["success"].as_bool().unwrap());
    let list = body["data"]["list"].as_array().unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0]["device_id"], device_names[0]);
    assert_eq!(list[0]["bound_reg_code_count"], 2);
    assert_eq!(list[1]["bound_reg_code_count"], 1);
    assert_eq!(list[2]["bound_reg_code_count"], 0);

    let bound_codes = list[0]["reg_codes"].as_array().unwrap();
    assert_eq!(bound_codes.len(), 2);
    assert!(bound_codes.iter().any(|item| item["code"] == code_names[0]));
    assert!(
        bound_codes
            .iter()
            .any(|item| item["code"] == code_names[1] && item["status"] == 4)
    );

    let response = TestClient::get(helpers::get_url(&format!(
        "/api/admin/devices/list?app_id={app_id}&sort_by=bound_reg_code_count&sort_order=asc&page=1&page_size=10"
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let body = print_response_body_get_json(response, "devices_by_code_count_asc").await;
    let list = body["data"]["list"].as_array().unwrap();
    assert_eq!(list[0]["bound_reg_code_count"], 0);
    assert_eq!(list[1]["bound_reg_code_count"], 1);
    assert_eq!(list[2]["bound_reg_code_count"], 2);
}
