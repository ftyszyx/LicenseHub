-
DROP TABLE IF EXISTS "apps" CASCADE;
CREATE TABLE "apps" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "app_id" VARCHAR(255) NOT NULL UNIQUE,
    "app_vername" VARCHAR(255) NOT NULL,
    "app_vercode" INTEGER NOT NULL,
    "app_download_url" VARCHAR(255) NOT NULL,
    "app_res_url" VARCHAR(255) NOT NULL,
    "app_update_info" TEXT,
    "app_valid_key" VARCHAR(255) NOT NULL DEFAULT '', -- 应用验证key
    "trial_days" INTEGER NOT NULL DEFAULT 0, -- 试用期时长
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "deleted_at" TIMESTAMPTZ,
    CONSTRAINT "chk_status_range" CHECK ("status" IN (0, 1))
);
COMMENT ON COLUMN "apps"."status" IS '0: 下架 1: 上架';
CREATE INDEX idx_apps_app_id ON "apps" ("app_id");

-- 设备表
DROP TABLE IF EXISTS "app_devices" CASCADE;
CREATE TABLE "app_devices" (
    "id" SERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "device_info" JSONB,
    "bind_time" TIMESTAMPTZ,
    "expire_time" TIMESTAMPTZ,
    CONSTRAINT "fk_app_device_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE INDEX idx_app_devices_app_id ON "app_devices" ("app_id");
CREATE INDEX idx_app_devices_device_id ON "app_devices" ("device_id");

-- 注册码
DROP TABLE IF EXISTS "reg_codes" CASCADE;
CREATE TABLE "reg_codes" (
    "id" SERIAL PRIMARY KEY,
    "code" VARCHAR NOT NULL UNIQUE,
    "app_id" INTEGER NOT NULL, -- 应用ID
    "valid_days" INTEGER NOT NULL DEFAULT 0, -- 有效天数 1: 1天 2: 3天 3: 7天 4: 30天
    "max_devices" INTEGER NOT NULL DEFAULT 1, -- 最大绑定设备数
    "status" SMALLINT NOT NULL DEFAULT 0, -- 状态 0: 未使用 1: 已使用 2: 已过期
    "binding_time" TIMESTAMPTZ, -- 绑定时间
    "code_type" SMALLINT NOT NULL DEFAULT 0, -- 0: 时间类型 1：次数类型
    "expire_time" TIMESTAMPTZ, -- 过期时间（时间类型）
    "total_count" INTEGER, -- 总次数（次数类型）
    "use_count" INTEGER NOT NULL DEFAULT 0, -- 已使用次数
    "device_id" INTEGER, -- 绑定设备ID
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "deleted_at" TIMESTAMPTZ,
    CONSTRAINT "fk_reg_code_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_reg_code_device_id" FOREIGN KEY ("device_id") REFERENCES "app_devices" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE INDEX idx_reg_codes_app_id ON "reg_codes" ("app_id");
CREATE INDEX idx_reg_codes_status ON "reg_codes" ("status");
