ALTER TABLE "orders"
    ADD COLUMN "provider_buyer_id" VARCHAR(255);

CREATE INDEX "idx_orders_provider_buyer_paid_at"
    ON "orders" ("provider", "provider_buyer_id", "paid_at" DESC)
    WHERE "provider_buyer_id" IS NOT NULL;

COMMENT ON COLUMN "orders"."provider_buyer_id" IS
    'Payment-provider buyer identifier, such as WeChat payer.openid or Alipay buyer_id';
