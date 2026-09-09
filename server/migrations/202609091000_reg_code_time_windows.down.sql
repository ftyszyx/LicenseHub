DROP INDEX IF EXISTS "idx_reg_codes_expire_time";

ALTER TABLE "reg_codes"
    DROP CONSTRAINT IF EXISTS "chk_reg_codes_time_window",
    DROP COLUMN IF EXISTS "expire_time",
    DROP COLUMN IF EXISTS "effective_time";

COMMENT ON COLUMN "reg_codes"."binding_time" IS '绑定时间';
