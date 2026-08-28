-- Old registration codes always used the legacy single-device flow. Preserve
-- that behavior before deriving the mode exclusively from max_devices.
UPDATE "reg_codes"
SET "max_devices" = 1
WHERE "multi_device_enabled" = FALSE;

-- Count-code balances now live exclusively on reg_codes. Clear any old
-- device-level count balance first so it cannot be counted twice.
UPDATE "app_devices" AS ad
SET "remaining" = 0,
    "updated_at" = CURRENT_TIMESTAMP
WHERE EXISTS (
    SELECT 1
    FROM "reg_codes" AS rc
    LEFT JOIN "reg_code_devices" AS rcd
        ON rcd."reg_code_id" = rc."id"
    WHERE rc."code_type" = 1
      AND rc."remaining_count" IS NULL
      AND (rc."device_id" = ad."id" OR rcd."device_id" = ad."id")
);

UPDATE "reg_codes"
SET "remaining_count" = CASE
    WHEN "status" IN (3, 4) THEN 0
    ELSE COALESCE("total_count", 0)
END
WHERE "code_type" = 1
  AND "remaining_count" IS NULL;

ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_count_remaining"
    CHECK (
        "code_type" <> 1
        OR ("remaining_count" IS NOT NULL AND "remaining_count" >= 0)
    );

ALTER TABLE "reg_codes"
    DROP COLUMN "multi_device_enabled";
