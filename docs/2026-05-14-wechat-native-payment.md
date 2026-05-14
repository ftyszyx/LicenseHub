# 2026-05-14 微信 Native 支付接入

## 支付架构

- 移除彩豆支付运行路径，不再使用 `/api/pay/caidou/*`。
- 新增独立 workspace crate：`server/crates/payment`。
- 支付 crate 暴露 `PaymentAdapter` trait、`PaymentRegistry`、统一下单请求/响应和支付通知结构。
- LicenseHub 后端只依赖 adapter 抽象，后续支付宝或其它渠道可新增 adapter 后注册到 registry。

## 微信 Native

- 新增 `WechatNativeAdapter`，支持微信支付 V3 Native 下单。
- 下单接口返回 `code_url`，后端保存到订单 `qr_code` 字段。
- 新增微信通知入口：`POST /api/pay/wechat/native/notify`。
- 通知处理包含微信支付 V3 平台签名验签、AES-256-GCM 资源解密、金额校验、订单幂等发码。

## 前端购买页

- `wechat_native` 不再打开新窗口。
- 下单后在订单弹窗内生成微信支付二维码。
- 付款完成后继续轮询订单状态并展示注册码。

## 配置

- 新增 `PAYMENT_PAY_TYPES=wechat_native`。
- 新增微信支付配置：`WECHAT_PAY_APP_ID`、`WECHAT_PAY_MCH_ID`、`WECHAT_PAY_MERCHANT_SERIAL_NO`、`WECHAT_PAY_MERCHANT_PRIVATE_KEY_PATH`、`WECHAT_PAY_API_V3_KEY`、`WECHAT_PAY_PLATFORM_PUBLIC_KEY_PATH` 等。
- 新增迁移 `202605141600_orders_provider_default_wechat`，将订单 provider 默认值改为 `wechat`。
