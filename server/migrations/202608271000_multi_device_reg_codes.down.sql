UPDATE "reg_codes" SET "status" = 1 WHERE "status" = 4;
ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2, 3));
COMMENT ON COLUMN "reg_codes"."status" IS
    '0: unused 1: issued 2: binded 3: refunded';

DROP TABLE IF EXISTS "reg_code_devices";

ALTER TABLE "reg_codes"
    DROP COLUMN "remaining_count",
    DROP COLUMN "multi_device_enabled";

ALTER TABLE "apps" DROP CONSTRAINT "chk_apps_max_devices";
ALTER TABLE "apps" DROP COLUMN "max_devices";

