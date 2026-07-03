UPDATE "reg_codes" SET "status" = 1 WHERE "status" = 3;
ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2));
