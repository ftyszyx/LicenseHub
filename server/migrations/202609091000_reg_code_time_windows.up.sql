ALTER TABLE "reg_codes"
    ADD COLUMN "effective_time" TIMESTAMPTZ,
    ADD COLUMN "expire_time" TIMESTAMPTZ;

-- Old multi-device codes may have a different expiry on each device. Keep the
-- latest existing expiry so upgrading never shortens an active authorization.
WITH bound_devices AS (
    SELECT
        rcd."reg_code_id",
        rcd."created_at" AS "binding_time",
        ad."expire_time"
    FROM "reg_code_devices" AS rcd
    INNER JOIN "app_devices" AS ad ON ad."id" = rcd."device_id"

    UNION ALL

    SELECT
        rc."id" AS "reg_code_id",
        COALESCE(rc."binding_time", rc."updated_at", rc."created_at") AS "binding_time",
        ad."expire_time"
    FROM "reg_codes" AS rc
    INNER JOIN "app_devices" AS ad ON ad."id" = rc."device_id"
), existing_windows AS (
    SELECT
        "reg_code_id",
        MIN("binding_time") AS "binding_time",
        MAX("expire_time") AS "expire_time"
    FROM bound_devices
    GROUP BY "reg_code_id"
)
UPDATE "reg_codes" AS rc
SET "binding_time" = COALESCE(rc."binding_time", ew."binding_time"),
    "expire_time" = ew."expire_time",
    "effective_time" = ew."expire_time" - make_interval(days => rc."valid_days")
FROM existing_windows AS ew
WHERE rc."id" = ew."reg_code_id"
  AND rc."code_type" = 0
  AND rc."status" = 2
  AND ew."expire_time" IS NOT NULL;

UPDATE "reg_codes"
SET "effective_time" = "binding_time",
    "expire_time" = "binding_time" + make_interval(days => "valid_days")
WHERE "code_type" = 0
  AND "binding_time" IS NOT NULL
  AND "expire_time" IS NULL;

ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_time_window"
    CHECK (
        "effective_time" IS NULL
        OR "expire_time" IS NULL
        OR "effective_time" <= "expire_time"
    );

CREATE INDEX "idx_reg_codes_expire_time" ON "reg_codes" ("expire_time");

COMMENT ON COLUMN "reg_codes"."binding_time" IS '注册码首次绑定时间';
COMMENT ON COLUMN "reg_codes"."effective_time" IS '时间型注册码权益生效时间';
COMMENT ON COLUMN "reg_codes"."expire_time" IS '时间型注册码统一过期时间';
