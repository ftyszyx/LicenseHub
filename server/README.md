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
cargo install sqlx-cli --version 0.8.6 --locked --no-default-features --features native-tls,postgres
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

支付能力已拆到 workspace crate `crates/payment`，业务层按数据库中的支付渠道配置动态创建 adapter。当前支持微信 Native 支付和支付宝电脑网站支付。

后端环境变量只保留支付总开关和站点地址：

```env
PAYMENT_ENABLED=true
PUBLIC_BASE_URL=https://你的后端域名
FRONTEND_BASE_URL=https://你的前端域名
PAYMENT_SITE_NAME=LicenseHub
```

Payment channels are configured in Admin > Payment Settings and stored in the database. Do not use
`PAYMENT_PAY_TYPES`, `WECHAT_PAY_ENABLED`, `ALIPAY_PAY_ENABLED`, or provider key/cert path env vars
for channel configuration anymore.

Create one or more channels in the admin UI. Currently supported providers are WeChat Pay Native and
Alipay PC website pay. WeChat merchant private keys, WeChat Pay public keys, Alipay app private keys,
and Alipay public keys are uploaded or pasted as PEM text and saved in the database. The WeChat merchant
private key is the full `apiclient_key.pem` content; the API v3 key is the 32-character string configured in
the WeChat Pay merchant platform. WeChat notification verification uses the new WeChat Pay public key
mode and requires the paired `wechatpay_public_key_id`.

Notify URL pattern:

```text
https://your-backend-domain/api/pay/{pay_type}/notify
```

Legacy notify URLs are still accepted for existing channels:

```text
https://your-backend-domain/api/pay/wechat/native/notify
https://your-backend-domain/api/pay/alipay/page/notify
```

Official docs:

- Alipay PC website pay: https://opendocs.alipay.com/open/270/105899
- Alipay key/sign docs: https://opendocs.alipay.com/common/02kipk
- WeChat Pay Native docs: https://pay.weixin.qq.com/doc/v3/merchant/4012791877
- WeChat Pay merchant docs: https://pay.weixin.qq.com/doc/v3/merchant/4013053053

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
