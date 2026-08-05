DELETE FROM "role_permissions"
WHERE "permission_id" IN (
    SELECT "id" FROM "permissions" WHERE "name" = 'distribution:read'
);

DELETE FROM "permissions" WHERE "name" = 'distribution:read';

DELETE FROM "system_settings" WHERE "key" IN (
    'distribution_enabled',
    'distribution_default_rate_bps',
    'distribution_attribution_days',
    'distribution_holding_days',
    'distribution_min_withdraw_cents'
);

DROP TABLE IF EXISTS "distribution_commissions";

DROP INDEX IF EXISTS "idx_orders_referrer_user_id";
ALTER TABLE "orders"
    DROP CONSTRAINT IF EXISTS "chk_orders_commission_amount_cents",
    DROP CONSTRAINT IF EXISTS "chk_orders_commission_rate_bps",
    DROP CONSTRAINT IF EXISTS "fk_orders_referrer_user_id",
    DROP COLUMN IF EXISTS "commission_amount_cents",
    DROP COLUMN IF EXISTS "commission_rate_bps",
    DROP COLUMN IF EXISTS "referral_code",
    DROP COLUMN IF EXISTS "referrer_user_id";

DROP INDEX IF EXISTS "uq_users_referral_code";
ALTER TABLE "users"
    DROP CONSTRAINT IF EXISTS "chk_users_commission_rate_bps",
    DROP COLUMN IF EXISTS "commission_rate_bps",
    DROP COLUMN IF EXISTS "referral_code";
