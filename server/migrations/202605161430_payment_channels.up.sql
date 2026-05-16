ALTER TABLE "orders" ALTER COLUMN "pay_type" TYPE VARCHAR(64);

CREATE TABLE "payment_channels" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "pay_type" VARCHAR(64) NOT NULL UNIQUE,
    "status" SMALLINT NOT NULL DEFAULT 1,
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "config" JSONB NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "chk_payment_channels_provider" CHECK ("provider" IN ('wechat', 'alipay')),
    CONSTRAINT "chk_payment_channels_status" CHECK ("status" IN (0, 1)),
    CONSTRAINT "chk_payment_channels_pay_type" CHECK (length(trim("pay_type")) > 0),
    CONSTRAINT "chk_payment_channels_name" CHECK (length(trim("name")) > 0)
);

CREATE INDEX idx_payment_channels_provider ON "payment_channels" ("provider");
CREATE INDEX idx_payment_channels_status_sort ON "payment_channels" ("status", "sort_order", "id");

SELECT setval(
    pg_get_serial_sequence('permissions', 'id'),
    COALESCE((SELECT MAX("id") FROM "permissions"), 1),
    true
);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('payment_settings:read', 'payment_settings', 'READ', 'Payment settings - read'),
    ('payment_settings:create', 'payment_settings', 'CREATE', 'Payment settings - create'),
    ('payment_settings:update', 'payment_settings', 'UPDATE', 'Payment settings - update'),
    ('payment_settings:delete', 'payment_settings', 'DELETE', 'Payment settings - delete')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p.name IN (
    'payment_settings:read',
    'payment_settings:create',
    'payment_settings:update',
    'payment_settings:delete'
)
AND NOT EXISTS (
    SELECT 1
    FROM "role_permissions" rp
    WHERE rp.role_id = 1 AND rp.permission_id = p.id
);
