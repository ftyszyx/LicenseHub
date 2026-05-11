CREATE UNIQUE INDEX IF NOT EXISTS uq_orders_provider_trade_no
ON "orders" ("provider", "provider_trade_no")
WHERE "provider_trade_no" IS NOT NULL;
