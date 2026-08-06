use app_server::core::app;
use app_server::core::router;
use once_cell::sync::Lazy;
use salvo::prelude::*;
use salvo::test::ResponseExt;
use salvo::test::TestClient;
use serde_json::{Value, json};
use std::env;
use std::sync::Once;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, MutexGuard};

static DB_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static LOG_ONCE: Once = Once::new();
// static LOG_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

pub struct TestContext {
    pub app: Service,
    #[allow(dead_code)]
    pub token: String,
    #[allow(dead_code)]
    pub app_state: app::AppState,
}

#[allow(dead_code)]
impl TestContext {
    pub async fn new() -> Self {
        create_test_context().await
    }

    pub async fn login_default_user(&mut self) {
        self.token = login_admin(&self.app).await;
    }

    pub fn bearer_token(&self) -> String {
        bearer(&self.token)
    }

    pub fn get_db_url(&self) -> String {
        self.app_state.config.database.db_url.clone()
    }
}

pub async fn create_test_context() -> TestContext {
    dotenvy::from_filename(".env.test").unwrap();
    LOG_ONCE.call_once(|| {
        let _ = app::init_log();
    });
    let app_state = app::init_app()
        .await
        .unwrap_or_else(|e| panic!("failed to initialize app:{}", e.to_string()));
    let pool = sqlx::PgPool::connect(&app_state.config.database.db_url)
        .await
        .unwrap();
    println!("clean database");
    assert!(
        app_state.config.database.db_name.ends_with("_test"),
        "refusing to reset a non-test database"
    );
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("CREATE SCHEMA public")
        .execute(&pool)
        .await
        .unwrap();
    println!("init database");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    println!("init database success");
    let app = router::create_router(app_state.clone());
    TestContext {
        app,
        token: String::new(),
        app_state: app_state,
    }
}

#[allow(dead_code)]
pub async fn create_test_app() -> Service {
    create_test_context().await.app
}

#[allow(dead_code)]
pub async fn db_lock() -> MutexGuard<'static, ()> {
    DB_MUTEX.lock().await
}

#[allow(dead_code)]
pub async fn seed_test_license_signing_key(ctx: &TestContext) {
    let pool = sqlx::PgPool::connect(&ctx.app_state.config.database.db_url)
        .await
        .expect("connect test database");
    sqlx::query(
        r#"
        insert into system_settings (key, value, created_at, updated_at)
        values ('license_signing_private_key_b64', $1, now(), now())
        on conflict (key) do update set value = excluded.value, updated_at = now()
        "#,
    )
    .bind("MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY")
    .execute(&pool)
    .await
    .expect("seed test license signing key");
}
pub async fn print_response_body_get_json(
    mut response: Response,
    label: &str,
) -> serde_json::Value {
    let status = response.status_code;
    let json = response.take_json().await.unwrap();
    println!("{}: status={:?}, body={:?}\n", label, status, json);
    json
}

#[allow(dead_code)]
pub async fn login_admin(app: &Service) -> String {
    let login_body = json!({
        "username": "admin",
        "password": "admin"
    });

    let url = get_url("/api/login");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&login_body)
        .send(app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "admin_login_response").await;
    json["data"]["token"].as_str().unwrap().to_string()
}

pub fn get_url(path: &str) -> String {
    let host = env::var("LISTEN_HOST").expect("LISTEN_HOST not set");
    let port = env::var("LISTEN_PORT").expect("LISTEN_PORT not set");
    if path.starts_with("/") {
        format!("http://{}:{}{}", host, port, path)
    } else {
        format!("http://{}:{}/{}", host, port, path)
    }
}

#[allow(dead_code)]
pub fn unique_name(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}", prefix, nanos)
}

#[allow(dead_code)]
pub fn bearer(token: &str) -> String {
    format!("Bearer {}", token)
}

#[allow(dead_code)]
pub async fn register_user(app: &Service, username: &str, password: &str) -> Value {
    let payload = json!({"username": username, "password": password});
    let response = TestClient::post(get_url("/api/register"))
        .add_header("content-type", "application/json", true)
        .json(&payload)
        .send(app)
        .await;
    print_response_body_get_json(response, "register").await
}

#[allow(dead_code)]
pub async fn login_user(app: &Service, username: &str, password: &str, label: &str) -> Value {
    let payload = json!({"username": username, "password": password});
    let response = TestClient::post(get_url("/api/login"))
        .add_header("content-type", "application/json", true)
        .json(&payload)
        .send(app)
        .await;
    print_response_body_get_json(response, label).await
}
