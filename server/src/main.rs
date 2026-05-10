use app_server::core::{app, router};
use salvo::prelude::*;
#[tokio::main]
async fn main() {
    // Load .env if present; do not fail when missing (use env vars instead)
    tracing::info!("Server1 booting");
    let _ = dotenvy::dotenv();
    let _guard = app::init_log();
    tracing::info!("Server boot starting");
    let app_state = app::init_app().await
        .unwrap_or_else(|e| panic!("failed to initialize app:{}", e.to_string()));
    let host = app_state.config.server.host.clone();
    let port = app_state.config.server.port;
    let app_service = router::create_router(app_state);
    // 启动服务器
    let addr=format!("{}:{}",host,port);
    tracing::info!("Server starting on {}", addr);
    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(app_service).await;
}
