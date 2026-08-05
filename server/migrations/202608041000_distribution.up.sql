ALTER TABLE "users"
    ADD COLUMN "referral_code" VARCHAR(32),
    ADD COLUMN "commission_rate_bps" INTEGER;

UPDATE "users"
SET "referral_code" = 'LH' || UPPER(SUBSTRING(MD5("id"::TEXT || CLOCK_TIMESTAMP()::TEXT) FROM 1 FOR 10))
WHERE "referral_code" IS NULL;

ALTER TABLE "users"
    ALTER COLUMN "referral_code" SET NOT NULL,
    ADD CONSTRAINT "chk_users_commission_rate_bps"
        CHECK ("commission_rate_bps" IS NULL OR "commission_rate_bps" BETWEEN 0 AND 10000);

CREATE UNIQUE INDEX "uq_users_referral_code" ON "users" ("referral_code");

ALTER TABLE "orders"
    ADD COLUMN "referrer_user_id" INTEGER,
    ADD COLUMN "referral_code" VARCHAR(32),
    ADD COLUMN "commission_rate_bps" INTEGER,
    ADD COLUMN "commission_amount_cents" INTEGER,
    ADD CONSTRAINT "fk_orders_referrer_user_id"
        FOREIGN KEY ("referrer_user_id") REFERENCES "users" ("id") ON DELETE SET NULL ON UPDATE CASCADE,
    ADD CONSTRAINT "chk_orders_commission_rate_bps"
        CHECK ("commission_rate_bps" IS NULL OR "commission_rate_bps" BETWEEN 0 AND 10000),
    ADD CONSTRAINT "chk_orders_commission_amount_cents"
        CHECK ("commission_amount_cents" IS NULL OR "commission_amount_cents" >= 0);

CREATE INDEX "idx_orders_referrer_user_id" ON "orders" ("referrer_user_id");

CREATE TABLE "distribution_commissions" (
    "id" BIGSERIAL PRIMARY KEY,
    "order_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "order_amount_cents" INTEGER NOT NULL,
    "commission_rate_bps" INTEGER NOT NULL,
    "commission_amount_cents" INTEGER NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "available_at" TIMESTAMPTZ,
    "cancel_reason" TEXT,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_commissions_order_id"
        FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_commissions_user_id"
        FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "uq_distribution_commissions_order_id" UNIQUE ("order_id"),
    CONSTRAINT "chk_distribution_commissions_amount" CHECK ("order_amount_cents" > 0),
    CONSTRAINT "chk_distribution_commissions_rate" CHECK ("commission_rate_bps" BETWEEN 0 AND 10000),
    CONSTRAINT "chk_distribution_commissions_commission" CHECK ("commission_amount_cents" >= 0),
    CONSTRAINT "chk_distribution_commissions_status" CHECK ("status" IN (0, 1, 4))
);

CREATE INDEX "idx_distribution_commissions_user_id" ON "distribution_commissions" ("user_id");
CREATE INDEX "idx_distribution_commissions_status_available_at"
    ON "distribution_commissions" ("status", "available_at");

INSERT INTO "system_settings" ("key", "value") VALUES
    ('distribution_enabled', 'false'),
    ('distribution_default_rate_bps', '2000'),
    ('distribution_attribution_days', '30'),
    ('distribution_holding_days', '7'),
    ('distribution_min_withdraw_cents', '5000')
ON CONFLICT ("key") DO NOTHING;

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('distribution:read', 'distribution', 'READ', 'Distribution commissions - read')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p."name" = 'distribution:read'
  AND NOT EXISTS (
      SELECT 1 FROM "role_permissions" rp
      WHERE rp."role_id" = 1 AND rp."permission_id" = p."id"
  );
