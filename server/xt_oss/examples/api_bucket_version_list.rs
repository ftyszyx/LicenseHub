//! cargo run --example api_bucket_version_list -q
//! 接口用于获取指定Bucket的版本控制状态。
//!
//! - [official docs](https://help.aliyun.com/zh/oss/developer-reference/getbucketversioning)
//! - [xtoss example](https://github.com/isme-sun/xt_oss/blob/main/examples/api_bucket_version_get.rs)
use dotenv;
use std::process;
use xt_oss::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    match util::options_from_env()
        .client()
        .ListObjectVersions()
        // .with_max_keys(5)
        // .with_prefix("mp3")
        .with_delimiter("/")
        // .with_key_marker(value)
        // .with_encoding_type(value)
        // .with_version_id_marker(value)
        .execute()
        .await
        .unwrap_or_else(|error| {
            println!("reqwest error {}", error);
            process::exit(-1);
        }) {
        Ok(data) => {
            let version_objects = data.content();
            // println!("{:#?}", version_objects);
            println!(
                "{}",
                serde_json::to_string_pretty(&version_objects).unwrap()
            );
        }
        Err(oss_error_message) => {
            println!("{:#?}", oss_error_message.content());
        }
    };
    Ok(())
}
