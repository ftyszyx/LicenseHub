use dotenv;
use std::{env, process};
use xt_oss::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let access_key_id = env::var("OSS_ACCESS_KEY_ID")?;
    let access_key_secret = env::var("OSS_ACCESS_KEY_SECRET")?;
    let url = "https://oss-cn-hangzhou.aliyuncs.com";

    let resp = oss::Request::new()
        .with_access_key_id(&access_key_id)
        .with_access_key_secret(&access_key_secret)
        .task()
        .with_url(&url)
        // default Method::GET
        // .with_method(http::Method::GET)
        .execute()
        .await
        .unwrap_or_else(|error| {
            println!("reqwest error: {}", error);
            process::exit(-1);
        });

    match resp.status().is_success() {
        true => println!("oss api sucess:"),
        false => println!("oss api fail:"),
    }

    println!("status: {}", resp.status());
    println!("headers: {:#?}", resp.headers());
    let data = resp.text().await?;
    println!("data: {}", data);
    Ok(())
}
