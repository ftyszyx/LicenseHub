use app_server::core::{app, constants, router};
use http_body_util::BodyExt;
use salvo::prelude::*;
use salvo::test::TestClient;
use serde_json::json;
use std::env;
use std::process::Command;

async fn ensure_test_database_exists() {
    // 读取测试环境的 DATABASE_URL，并从中提取数据库名
    let database_url = env::var("DB_URL").expect("DB_URL must be set in .env.test");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set in .env.test");
    // 使用 psql 连接到 postgres 管理库，检查数据库是否存在
    let check_sql = format!(
        "SELECT 1 FROM pg_database WHERE datname = '{}' LIMIT 1;",
        escape_sql_literal(&db_name)
    );
    let output = Command::new("psql")
        .env("PGCLIENTENCODING", "UTF8")
        .arg(&database_url)
        .args(["-tA", "-q", "-v", "ON_ERROR_STOP=1"]) // 仅输出值，安静模式
        .args(["-c", &check_sql])
        .output()
        .expect("failed to run psql for database existence check");
    if !output.status.success() {
        panic!(
            "psql check failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let exists = String::from_utf8_lossy(&output.stdout)
        .trim()
        .starts_with('1');
    if !exists {
        println!("database {} not exists, creating...", db_name);
        let create_sql = format!(
            "CREATE DATABASE \"{}\" WITH TEMPLATE template0 ENCODING 'UTF8'",
            db_name
        );
        let out = Command::new("psql")
            .env("PGCLIENTENCODING", "UTF8")
            .arg(&database_url)
            .args(["-q", "-v", "ON_ERROR_STOP=1"])
            .args(["-c", &create_sql])
            .output()
            .expect("failed to run psql to create database");
        if !out.status.success() {
            panic!(
                "psql create db failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    } else {
        println!("database {} exists", db_name);
    }
}

fn escape_sql_literal(s: &str) -> String {
    s.replace("'", "''")
}

fn run_init_sql_with_psql() {
    println!("running init.sql...");
    println!("cur dir:{}", env::current_dir().unwrap().display());
    let db_name = env::var("DB_NAME").expect("DB_NAME not set");
    let database_url = env::var("DB_URL").expect("DB_URL not set");
    let init_sql_file = format!("../pub/deploy/postgres/init/init.sql");
    let connect_url = format!("{}/{}", database_url, db_name);
    println!("connect_url:{}", connect_url);
    let output = Command::new("psql")
        .env("PGCLIENTENCODING", "UTF8")
        .arg(&connect_url)
        .args(["-v", "ON_ERROR_STOP=1", "-q"]) // 安静模式，失败即停止
        .args(["-c", "SET client_min_messages = warning;"]) // 隐藏 NOTICE
        .args(["-f", &init_sql_file])
        .output()
        .expect("failed to spawn psql");
    if !output.status.success() {
        panic!(
            "psql init.sql failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!(
        "init.sql completed:{}",
        String::from_utf8_lossy(&output.stdout)
    );
}


pub async fn create_test_app() -> Service {
    dotenvy::from_filename(".env.test").unwrap();
    let _guard = app::init_log();
    // 确保测试数据库存在（若不存在则创建），再初始化应用
    ensure_test_database_exists().await;
    run_init_sql_with_psql();
    let app_state = app::init_app()
        .await
        .unwrap_or_else(|e| panic!("failed to initialize app:{}", e.to_string()));
    let app = router::create_router(app_state);
    app
}

pub async fn print_response_body_get_json(response: Response, label: &str) -> serde_json::Value {
    let status = response.status_code;
    let body = response.body.collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    println!("{}: status={:?}, body={}\n", label, status, json);
    json
}

#[allow(dead_code)]
pub async fn create_test_user_and_login(app: &Service) -> String {
    // 注册用户
    let register_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });
    let url=get_url("/api/register");
    let response = TestClient::post(url)
        .add_header("content-type", "application/json", true)
        .json(&register_body)
        .send(app)
        .await;

    println!("register_body: {:?}", register_body);
    let json = print_response_body_get_json(response, "register_response").await;
    let code = json["code"].as_u64().unwrap();
    assert!(code == 0 || code == constants::APP_USER_ALREADY_EXISTS as u64);

    // 登录获取 token
    let login_body = json!({
        "username": "testuser",
        "password": "testpass123"
    });

    let url=get_url("/api/login");
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
