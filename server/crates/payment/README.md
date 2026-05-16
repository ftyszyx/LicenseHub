# Payment crate

Payment providers are exposed through the `PaymentAdapter` trait so the app server can create orders and parse provider notifications through one boundary.

## WeChat Native

`WechatNativeAdapter` supports WeChat Pay V3 Native order creation and payment notifications.
It uses the new WeChat Pay public key mode for notification signature verification, so callers must
provide both `wechatpay_public_key_id` and `wechatpay_public_key_pem`. The merchant private key is
the full `apiclient_key.pem` PEM content; `api_v3_key` is the separate 32-character string used for
resource decryption.

## Alipay Page Pay

`AlipayPageAdapter` supports Alipay PC website payment through `alipay.trade.page.pay`. It generates a signed gateway URL for browser redirection and verifies Alipay asynchronous notifications.
