# Payment crate

Payment providers are exposed through the `PaymentAdapter` trait so the app server can create orders and parse provider notifications through one boundary.

## WeChat Native

`WechatNativeAdapter` supports WeChat Pay V3 Native order creation and payment notifications.

## Alipay Page Pay

`AlipayPageAdapter` supports Alipay PC website payment through `alipay.trade.page.pay`. It generates a signed gateway URL for browser redirection and verifies Alipay asynchronous notifications.
