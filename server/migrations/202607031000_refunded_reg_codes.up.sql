ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2, 3));
COMMENT ON COLUMN "reg_codes"."status" IS '0: unused 1: issued 2: binded 3: refunded';
