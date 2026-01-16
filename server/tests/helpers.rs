use app_server::core::app;
use app_server::core::constants::*;
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

pub async fn create_test_app() -> Service {
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
    sqlx::migrate!("./migrations").undo(&pool, 0).await.unwrap();
    println!("init database");
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    println!("init database success");
    let app = router::create_router(app_state);
    app
}

#[allow(dead_code)]
pub async fn db_lock() -> MutexGuard<'static, ()> {
    DB_MUTEX.lock().await
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
pub async fn create_test_user_and_login(app: &Service) -> String {
    // 注册用户
    let register_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });
    let url = get_url("/api/register");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&register_body)
        .send(app)
        .await;

    println!("register_body: {:?}", register_body);
    let json = print_response_body_get_json(response, "register_response").await;
    let code = json["code"].as_u64().unwrap();
    assert!(code == 0 || code == APP_USER_ALREADY_EXISTS as u64);

    // 登录获取 token
    let login_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });

    let url = get_url("/api/login");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&login_body)
        .send(app)
        .await;
    assert_eq!(response.status_code, Some(StatusCode::OK));
    let json = print_response_body_get_json(response, "login_response").await;
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
