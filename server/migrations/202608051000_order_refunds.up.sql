ALTER TABLE "orders"
    DROP CONSTRAINT "chk_orders_status",
    ADD CONSTRAINT "chk_orders_status" CHECK ("status" IN (0, 1, 2, 3, 4, 5));

CREATE TABLE "order_refunds" (
    "id" BIGSERIAL PRIMARY KEY,
    "refund_no" VARCHAR(64) NOT NULL UNIQUE,
    "order_id" INTEGER NOT NULL UNIQUE,
    "amount_cents" INTEGER NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "provider_trade_no" VARCHAR(128),
    "refund_reference" VARCHAR(255) NOT NULL,
    "reason" TEXT NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 1,
    "operator_user_id" INTEGER NOT NULL,
    "refunded_at" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_order_refunds_order_id"
        FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_order_refunds_operator_user_id"
        FOREIGN KEY ("operator_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_order_refunds_amount" CHECK ("amount_cents" > 0),
    CONSTRAINT "chk_order_refunds_status" CHECK ("status" IN (0, 1, 2))
);

CREATE INDEX "idx_order_refunds_operator_user_id" ON "order_refunds" ("operator_user_id");
CREATE INDEX "idx_order_refunds_refunded_at" ON "order_refunds" ("refunded_at" DESC);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('orders:read', 'orders', 'READ', 'Orders - read'),
    ('orders:update', 'orders', 'UPDATE', 'Orders - refund confirmation')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p."id"
FROM "permissions" p
WHERE p."name" IN ('orders:read', 'orders:update')
  AND NOT EXISTS (
      SELECT 1
      FROM "role_permissions" rp
      WHERE rp."role_id" = 1 AND rp."permission_id" = p."id"
  );
