DROP INDEX IF EXISTS "idx_orders_provider_buyer_paid_at";

ALTER TABLE "orders"
    DROP COLUMN IF EXISTS "provider_buyer_id";
