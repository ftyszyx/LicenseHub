# 支付模块

支付能力通过 adapter 暴露，业务系统只依赖统一 trait。

## 微信 Native 支付

已实现 `WechatNativeAdapter`，对接微信支付 V3 Native 下单和支付通知。

## 支付宝支付

预留 adapter 扩展位。后续新增支付宝时，在本 crate 中实现 `PaymentAdapter` 并注册到业务系统的 `PaymentRegistry`。
