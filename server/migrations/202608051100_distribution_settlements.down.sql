DELETE FROM "role_permissions"
WHERE "permission_id" IN (
    SELECT "id" FROM "permissions" WHERE "name" = 'distribution:update'
);

DELETE FROM "permissions" WHERE "name" = 'distribution:update';

DROP TABLE IF EXISTS "distribution_commission_adjustment_offsets";
DROP TABLE IF EXISTS "distribution_commission_adjustments";
DROP TABLE IF EXISTS "distribution_settlement_proofs";
DROP TABLE IF EXISTS "distribution_settlement_items";
DROP TABLE IF EXISTS "distribution_settlements";

ALTER TABLE "distribution_commissions"
    DROP CONSTRAINT IF EXISTS "chk_distribution_commissions_allocated_amount";

UPDATE "distribution_commissions"
SET "status" = CASE
    WHEN "cancelled_amount_cents" = "commission_amount_cents" THEN 4
    ELSE 1
END
WHERE "status" IN (2, 3, 5);

ALTER TABLE "distribution_commissions"
    DROP CONSTRAINT "chk_distribution_commissions_status",
    ADD CONSTRAINT "chk_distribution_commissions_status" CHECK ("status" IN (0, 1, 4)),
    DROP COLUMN "adjustment_amount_cents",
    DROP COLUMN "cancelled_amount_cents",
    DROP COLUMN "settled_amount_cents",
    DROP COLUMN "locked_amount_cents";

ALTER TABLE "users"
    DROP COLUMN "settlement_account";
