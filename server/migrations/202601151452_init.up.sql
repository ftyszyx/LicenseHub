-- 角色
CREATE TABLE "roles" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR NOT NULL UNIQUE,
    "description" TEXT,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
INSERT INTO "roles" ( "name", "description") VALUES ( 'admin', '管理员');
INSERT INTO "roles" ( "name", "description") VALUES ( 'user', '用户');
INSERT INTO "roles" ( "name", "description") VALUES ( 'guest', '访客');

-- 用户
CREATE TABLE "users" (
    "id" SERIAL PRIMARY KEY,
    "username" VARCHAR(255) NOT NULL UNIQUE,
    "password" VARCHAR(255) NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
INSERT INTO "users" ( "username", "password") VALUES ( 'admin', '$2b$12$/MZyRsK.DcYHh6x4qCy6IOjxO/Wd4RlPSbW.7OiAYqTY4U4CipDIS');


-- Permissions Table
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT "action_check" CHECK (action IN ('READ', 'CREATE', 'UPDATE', 'DELETE','*'))
);

INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (1, 'all', '*', '*', '所有资源');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (2, 'users:read', 'users', 'READ', '用户-读取');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (3, 'users:create', 'users', 'CREATE', '用户-创建');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (4, 'users:update', 'users', 'UPDATE', '用户-更新');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (5, 'users:delete', 'users', 'DELETE', '用户-删除');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (6, 'roles:read', 'roles', 'READ', '角色-读取');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (7, 'roles:create', 'roles', 'CREATE', '角色-创建');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (8, 'roles:update', 'roles', 'UPDATE', '角色-更新');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (9, 'roles:delete', 'roles', 'DELETE', '角色-删除');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (10, 'apps:read', 'apps', 'READ', '应用-读取');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (11, 'apps:create', 'apps', 'CREATE', '应用-创建');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (12, 'apps:update', 'apps', 'UPDATE', '应用-更新');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (13, 'apps:delete', 'apps', 'DELETE', '应用-删除');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (14, 'reg_codes:read', 'reg_codes', 'READ', '注册码-读取');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (15, 'reg_codes:create', 'reg_codes', 'CREATE', '注册码-创建');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (16, 'reg_codes:update', 'reg_codes', 'UPDATE', '注册码-更新');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (17, 'reg_codes:delete', 'reg_codes', 'DELETE', '注册码-删除');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (18, 'devices:read', 'devices', 'READ', '设备-读取');
INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (19, 'me:update', 'me', 'UPDATE', '修改自己的密码');
-- User_Roles Join Table
CREATE TABLE user_roles (
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);

INSERT INTO "user_roles" ( "user_id", "role_id") VALUES ( 1, 1);

-- Role_Permissions Join Table
CREATE TABLE role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

INSERT INTO "role_permissions" ( "role_id", "permission_id") VALUES ( 1, 1);
INSERT INTO "role_permissions" ( "role_id", "permission_id") VALUES ( 2, 10);
INSERT INTO "role_permissions" ( "role_id", "permission_id") VALUES ( 2, 14);
INSERT INTO "role_permissions" ( "role_id", "permission_id") VALUES ( 2, 19);

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
    "app_valid_key" VARCHAR(255) NOT NULL DEFAULT '', -- 应用验证key
    "trial_days" INTEGER NOT NULL DEFAULT 0, -- 试用期时长
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
    "bind_time" TIMESTAMPTZ,
    "expire_time" TIMESTAMPTZ,
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
    "status" SMALLINT NOT NULL DEFAULT 0, -- 状态 0: 未使用 1: 已使用 2: 已过期
    "binding_time" TIMESTAMPTZ, -- 绑定时间
    "code_type" SMALLINT NOT NULL DEFAULT 0, -- 0: 时间类型 1：次数类型
    "expire_time" TIMESTAMPTZ, -- 过期时间（时间类型）
    "total_count" INTEGER, -- 总次数（次数类型）
    "use_count" INTEGER NOT NULL DEFAULT 0, -- 已使用次数
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
