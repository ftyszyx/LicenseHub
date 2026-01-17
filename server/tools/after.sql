
-- 产品表
CREATE TABLE "apps" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "app_id" VARCHAR(255) NOT NULL UNIQUE,
    "app_vername" VARCHAR(255) NOT NULL,
    "app_vercode" INTEGER NOT NULL,
    "app_download_url" VARCHAR(255) NOT NULL,
    "app_res_url" VARCHAR(255) NOT NULL,
    "app_update_info" TEXT,
     "code_type" SMALLINT NOT NULL DEFAULT 0, -- 0: 时间类型 1：次数类型
    "app_valid_key" VARCHAR(255) NOT NULL DEFAULT '', -- 应用验证key
    "trial_days" INTEGER NOT NULL DEFAULT 0, -- 试用期时长
    "trial_num" INTEGER NOT NULL DEFAULT 0, -- 试用次数
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "chk_status_range" CHECK ("status" IN (0, 1))
);
COMMENT ON COLUMN "apps"."status" IS '0: 下架 1: 上架';

-- 设备表
CREATE TABLE "app_devices" (
    "id" SERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "device_info" JSONB,
    "expire_time" TIMESTAMPTZ,-- 最终过期时间(时间类型)
    "remaining" INTEGER, -- 剩余次数（次数类型）
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_app_device_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uq_app_devices_app_id_device_id" UNIQUE ("app_id", "device_id")
);
CREATE INDEX idx_app_devices_app_id ON "app_devices" ("app_id");
CREATE INDEX idx_app_devices_device_id ON "app_devices" ("device_id");

-- 注册码
CREATE TABLE "reg_codes" (
    "id" SERIAL PRIMARY KEY,
    "code" VARCHAR NOT NULL UNIQUE,
    "app_id" INTEGER NOT NULL, -- 应用ID
    "valid_days" INTEGER NOT NULL DEFAULT 0, -- 有效天数 1: 1天 2: 3天 3: 7天 4: 30天
    "max_devices" INTEGER NOT NULL DEFAULT 1, -- 最大绑定设备数
    "status" SMALLINT NOT NULL DEFAULT 0, -- 状态 0: 未使用 1: 已使用(未绑定) 2: 已绑定
    "binding_time" TIMESTAMPTZ, -- 绑定时间
    "code_type" SMALLINT NOT NULL DEFAULT 0, -- 0: 时间类型 1：次数类型
    "total_count" INTEGER, -- 总次数（次数类型）
    "device_id" INTEGER, -- 绑定设备ID
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_reg_code_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_reg_code_device_id" FOREIGN KEY ("device_id") REFERENCES "app_devices" ("id") ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2)),
    CONSTRAINT "chk_reg_codes_code_type" CHECK ("code_type" IN (0, 1))
);
CREATE INDEX idx_reg_codes_app_id ON "reg_codes" ("app_id");
CREATE INDEX idx_reg_codes_status ON "reg_codes" ("status");