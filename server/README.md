# intro

# build and run

```
cargo build
cargo run
```

# test

test all

```
cargo test -- --test-threads=1
```

test apptest

```
cargo test --test app_tests -- --test-threads=1
cargo test --test reg_codes_tests -- --test-threads=1
cargo test --test resources_tests -- --test-threads=1
cargo test --test role_tests -- --test-threads=1
cargo test --test user_tests -- --test-threads=1
```

### 数据库迁移

```
cargo install sqlx-cli
```

数据库迁移

```bash
sqlx migrate run --database-url postgres://test:123456@localhost:5432/hub
```

清除

```
sqlx migrate revert --database-url postgres://test:123456@localhost:5432/hub
sqlx migrate revert --target-version 0 --database-url postgres://test:123456@localhost:5432/hub

```

## 支付接入

支付能力已拆到 workspace crate `crates/payment`，业务层通过 adapter 调用具体渠道。当前内置 `wechat_native`，后续接支付宝或其它渠道时优先在该 crate 中新增 adapter，再在服务端注册。

微信 Native 支付需要配置：

```env
PAYMENT_ENABLED=true
PAYMENT_PAY_TYPES=wechat_native
PUBLIC_BASE_URL=https://你的后端域名
FRONTEND_BASE_URL=https://你的前端域名
PAYMENT_SITE_NAME=LicenseHub
WECHAT_PAY_ENABLED=true
WECHAT_PAY_APP_ID=
WECHAT_PAY_MCH_ID=
WECHAT_PAY_MERCHANT_SERIAL_NO=
WECHAT_PAY_MERCHANT_PRIVATE_KEY_PATH=/path/to/apiclient_key.pem
WECHAT_PAY_API_V3_KEY=
WECHAT_PAY_PLATFORM_PUBLIC_KEY_PATH=/path/to/wechatpay_public_key.pem
WECHAT_PAY_API_BASE_URL=https://api.mch.weixin.qq.com
```

微信回调地址为：

```text
https://你的后端域名/api/pay/wechat/native/notify
```

### 生成entity

```
cargo install sea-orm-cli
```

```
sea-orm-cli generate entity -u "postgres://test:123456@localhost:5432/hub" -o "crates/data_model/src" --with-serde both
```

## docker发布

## 一些命令

### build docker

```
docker compose build server
```

### test docker

```
如果有 bash：docker compose run --rm -it server bash
```

#### 在服务器上执行

```
update_server.sh
```
