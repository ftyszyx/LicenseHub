use salvo::test::TestClient;
use serde_json::json;

mod helpers;

async fn create_app(ctx: &helpers::TestContext) -> i32 {
    let body = json!({
        "name": helpers::unique_name("SyncApp"),
        "app_id": helpers::unique_name("sync_app"),
        "app_vername": "2.1.0",
        "app_vercode": 210,
        "app_download_url": "https://example.com/download/app.exe",
        "app_res_url": "https://example.com/download/res.zip",
        "app_update_info": "Add version sync test coverage",
        "manifest_extra": {
            "channel": "stable",
            "force_update": true,
            "min_version_code": 200
        },
        "app_valid_key": helpers::unique_name("SYNC_KEY"),
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });

    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "version_sync_create_app").await;
    assert!(json["success"].as_bool().unwrap());
    json["data"]["id"].as_i64().unwrap() as i32
}

async fn create_storage_channel(ctx: &helpers::TestContext, provider: &str, status: i16) -> i32 {
    let body = json!({
        "name": helpers::unique_name("MockStorage"),
        "provider": provider,
        "status": status,
        "sort_order": 1,
        "config": {
            "bucket": "licensehub-test",
            "region": "test-region",
            "endpoint": "mock://storage",
            "access_key_id": "test-ak",
            "access_key_secret": "test-sk",
            "public_base_url": "cdn.example.com",
            "prefix": "apps"
        }
    });

    let resp = TestClient::post(helpers::get_url("/api/admin/storage-channels"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "create_storage_channel").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["provider"].as_str().unwrap(), provider);
    assert_eq!(json["data"]["config"]["prefix"].as_str().unwrap(), "apps");
    if provider == "cloudflare_r2" {
        assert_eq!(
            json["data"]["config"]["public_base_url"].as_str().unwrap(),
            "https://cdn.example.com"
        );
    }
    json["data"]["id"].as_i64().unwrap() as i32
}

#[tokio::test]
async fn test_storage_channels_crud() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let channel_id = create_storage_channel(&ctx, "cloudflare_r2", 1).await;

    let resp = TestClient::get(helpers::get_url(
        "/api/admin/storage-channels/list?page=1&page_size=20&provider=cloudflare_r2&status=1",
    ))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "list_storage_channels").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(
        json["data"]["list"]
            .as_array()
            .unwrap()
            .iter()
            .any(|item| item["id"].as_i64() == Some(channel_id as i64))
    );

    let resp = TestClient::put(helpers::get_url(&format!(
        "/api/admin/storage-channels/{}",
        channel_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({
        "name": "Disabled mock channel",
        "status": 0,
        "sort_order": 9,
        "config": {
            "bucket": "licensehub-test",
            "region": "",
            "endpoint": "mock://updated-storage",
            "access_key_id": "test-ak",
            "access_key_secret": "test-sk",
            "public_base_url": "https://cdn2.example.com/",
            "prefix": "/apps/"
        }
    }))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "update_storage_channel").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["status"].as_i64().unwrap(), 0);
    assert_eq!(json["data"]["sort_order"].as_i64().unwrap(), 9);
    assert_eq!(json["data"]["config"]["prefix"].as_str().unwrap(), "apps");

    let resp = TestClient::post(helpers::get_url("/api/admin/storage-channels"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&json!({
            "name": "Aliyun endpoint-only channel",
            "provider": "aliyun_oss",
            "status": 1,
            "sort_order": 2,
            "config": {
                "bucket": "licensehub-test",
                "endpoint": "oss-cn-hangzhou.aliyuncs.com",
                "access_key_id": "test-ak",
                "access_key_secret": "test-sk",
                "public_base_url": "",
                "prefix": "apps",
                "storage_class": "ia",
                "object_acl": "public-read"
            }
        }))
        .send(&ctx.app)
        .await;
    let json =
        helpers::print_response_body_get_json(resp, "create_aliyun_endpoint_only_channel").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(
        json["data"]["config"]["endpoint"].as_str().unwrap(),
        "https://oss-cn-hangzhou.aliyuncs.com"
    );
    assert_eq!(
        json["data"]["config"]["public_base_url"].as_str().unwrap(),
        "https://licensehub-test.oss-cn-hangzhou.aliyuncs.com"
    );
    assert_eq!(
        json["data"]["config"]["storage_class"].as_str().unwrap(),
        "ia"
    );
    assert_eq!(
        json["data"]["config"]["object_acl"].as_str().unwrap(),
        "public-read"
    );

    let resp = TestClient::delete(helpers::get_url(&format!(
        "/api/admin/storage-channels/{}",
        channel_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "delete_storage_channel").await;
    assert!(json["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_version_manifest_preview_and_manual_sync_logs() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let app_id = create_app(&ctx).await;
    let aliyun_channel_id = create_storage_channel(&ctx, "aliyun_oss", 1).await;
    let r2_channel_id = create_storage_channel(&ctx, "cloudflare_r2", 1).await;
    let disabled_channel_id = create_storage_channel(&ctx, "cloudflare_r2", 0).await;

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/admin/apps/{}/version-manifest",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "version_manifest_preview").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["schema_version"].as_i64().unwrap(), 1);
    assert_eq!(json["data"]["version"]["code"].as_i64().unwrap(), 210);
    assert_eq!(
        json["data"]["download_url"].as_str().unwrap(),
        "https://example.com/download/app.exe"
    );
    assert_eq!(
        json["data"]["res_url"].as_str().unwrap(),
        "https://example.com/download/res.zip"
    );
    assert_eq!(json["data"]["extra"]["channel"].as_str().unwrap(), "stable");
    assert_eq!(
        json["data"]["extra"]["force_update"].as_bool().unwrap(),
        true
    );

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/apps/{}/sync-version",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({ "channel_ids": [aliyun_channel_id, r2_channel_id] }))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "sync_selected_channels").await;
    assert!(json["success"].as_bool().unwrap());
    assert_eq!(json["data"]["results"].as_array().unwrap().len(), 2);
    assert!(
        json["data"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["success"].as_bool() == Some(true))
    );
    assert!(
        json["data"]["results"]
            .as_array()
            .unwrap()
            .iter()
            .all(|item| item["etag"].as_str() == Some("mock-etag"))
    );

    let resp = TestClient::post(helpers::get_url(&format!(
        "/api/admin/apps/{}/sync-version",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .add_header("content-type", "application/json", true)
    .json(&json!({}))
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "sync_all_enabled_channels").await;
    assert!(json["success"].as_bool().unwrap());
    let results = json["data"]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);
    assert!(
        results
            .iter()
            .all(|item| item["channel_id"].as_i64() != Some(disabled_channel_id as i64))
    );

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/admin/version-sync-logs?page=1&page_size=20&app_id={}",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "list_version_sync_logs").await;
    assert!(json["success"].as_bool().unwrap());
    let logs = json["data"]["list"].as_array().unwrap();
    assert_eq!(logs.len(), 4);
    assert!(logs.iter().any(|item| {
        item["storage_channel_id"].as_i64() == Some(aliyun_channel_id as i64)
            && item["status"].as_i64() == Some(1)
            && item["object_key"]
                .as_str()
                .is_some_and(|key| key.ends_with("/latest.json"))
    }));

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/admin/apps/list?page=1&page_size=20&id={}",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "list_apps_with_manifest_url").await;
    assert!(json["success"].as_bool().unwrap());
    let apps = json["data"]["list"].as_array().unwrap();
    assert_eq!(apps.len(), 1);
    let manifest_urls = apps[0]["manifest_urls"].as_array().unwrap();
    assert_eq!(manifest_urls.len(), 2);
    assert!(manifest_urls.iter().all(|item| {
        item["public_url"]
            .as_str()
            .is_some_and(|url| url.ends_with("/latest.json"))
    }));
    assert!(
        manifest_urls
            .iter()
            .any(|item| item["channel_id"].as_i64() == Some(aliyun_channel_id as i64))
    );
    assert!(
        manifest_urls
            .iter()
            .any(|item| item["channel_id"].as_i64() == Some(r2_channel_id as i64))
    );
}

#[tokio::test]
async fn test_version_manifest_omits_empty_res_url_and_extra() {
    let _lock = helpers::db_lock().await;
    let mut ctx = helpers::create_test_context().await;
    ctx.login_default_user().await;

    let body = json!({
        "name": helpers::unique_name("MinimalSyncApp"),
        "app_id": helpers::unique_name("minimal_sync_app"),
        "app_vername": "1.0.0",
        "app_vercode": 1,
        "app_download_url": "https://example.com/download/app.exe",
        "app_res_url": "",
        "app_update_info": "",
        "app_valid_key": helpers::unique_name("MINIMAL_SYNC_KEY"),
        "trial_days": 0,
        "sort_order": 0,
        "status": 1
    });

    let resp = TestClient::post(helpers::get_url("/api/admin/apps"))
        .add_header("authorization", helpers::bearer(&ctx.token), true)
        .add_header("content-type", "application/json", true)
        .json(&body)
        .send(&ctx.app)
        .await;
    let json = helpers::print_response_body_get_json(resp, "minimal_version_sync_create_app").await;
    assert!(json["success"].as_bool().unwrap());
    let app_id = json["data"]["id"].as_i64().unwrap() as i32;

    let resp = TestClient::get(helpers::get_url(&format!(
        "/api/admin/apps/{}/version-manifest",
        app_id
    )))
    .add_header("authorization", helpers::bearer(&ctx.token), true)
    .send(&ctx.app)
    .await;
    let json = helpers::print_response_body_get_json(resp, "minimal_version_manifest").await;
    assert!(json["success"].as_bool().unwrap());
    assert!(json["data"].get("res_url").is_none());
    assert!(json["data"].get("extra").is_none());
}
