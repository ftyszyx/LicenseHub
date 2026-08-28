ALTER TABLE "reg_codes"
    DROP CONSTRAINT "chk_reg_codes_count_remaining";

ALTER TABLE "reg_codes"
    ADD COLUMN "multi_device_enabled" BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE "reg_codes"
SET "multi_device_enabled" = ("max_devices" > 1);
