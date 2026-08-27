ALTER TABLE "apps"
    ADD COLUMN "max_devices" INTEGER NOT NULL DEFAULT 1;

ALTER TABLE "apps"
    ADD CONSTRAINT "chk_apps_max_devices" CHECK ("max_devices" >= 1);

ALTER TABLE "reg_codes"
    ADD COLUMN "multi_device_enabled" BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN "remaining_count" INTEGER;

CREATE TABLE "reg_code_devices" (
    "reg_code_id" INTEGER NOT NULL,
    "device_id" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY ("reg_code_id", "device_id"),
    CONSTRAINT "fk_reg_code_devices_reg_code_id"
        FOREIGN KEY ("reg_code_id") REFERENCES "reg_codes" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_reg_code_devices_device_id"
        FOREIGN KEY ("device_id") REFERENCES "app_devices" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE INDEX "idx_reg_code_devices_device_id"
    ON "reg_code_devices" ("device_id");

-- Preserve every existing binding as a legacy, single-device authorization.
INSERT INTO "reg_code_devices" ("reg_code_id", "device_id", "created_at")
SELECT "id", "device_id", COALESCE("binding_time", "updated_at", "created_at")
FROM "reg_codes"
WHERE "device_id" IS NOT NULL
ON CONFLICT ("reg_code_id", "device_id") DO NOTHING;

-- Status 4 (revoked) is already used by the application. Add it to the
-- database constraint while this migration is touching registration codes.
ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2, 3, 4));
COMMENT ON COLUMN "reg_codes"."status" IS
    '0: unused 1: issued 2: binded 3: refunded 4: revoked';

