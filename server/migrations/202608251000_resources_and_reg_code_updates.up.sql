-- 通用资源表及资源管理权限
CREATE TABLE "resources" (
    "id" BIGSERIAL PRIMARY KEY,
    "storage_channel_id" INTEGER NOT NULL,
    "object_key" VARCHAR(512) NOT NULL UNIQUE,
    "resource_type" VARCHAR(64) NOT NULL,
    "original_name" VARCHAR(255) NOT NULL,
    "content_type" VARCHAR(128) NOT NULL,
    "size" BIGINT NOT NULL,
    "uploaded_by" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_resources_storage_channel_id"
        FOREIGN KEY ("storage_channel_id") REFERENCES "storage_channels" ("id")
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_resources_uploaded_by"
        FOREIGN KEY ("uploaded_by") REFERENCES "users" ("id")
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_resources_resource_type"
        CHECK (length(trim("resource_type")) > 0),
    CONSTRAINT "chk_resources_object_key"
        CHECK (length(trim("object_key")) > 0),
    CONSTRAINT "chk_resources_size"
        CHECK ("size" > 0 AND "size" <= 20971520)
);

CREATE INDEX "idx_resources_type_created_at"
    ON "resources" ("resource_type", "created_at" DESC);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('resources:read', 'resources', 'READ', 'Resources - read'),
    ('resources:create', 'resources', 'CREATE', 'Resources - create'),
    ('resources:delete', 'resources', 'DELETE', 'Resources - delete')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p.name IN ('resources:read', 'resources:create', 'resources:delete')
AND NOT EXISTS (
    SELECT 1 FROM "role_permissions" rp
    WHERE rp.role_id = 1 AND rp.permission_id = p.id
);

-- 退款附件关联通用资源
CREATE TABLE "order_refund_attachments" (
    "refund_id" BIGINT PRIMARY KEY,
    "resource_id" BIGINT NOT NULL UNIQUE,
    "uploaded_by" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_order_refund_attachments_refund_id"
        FOREIGN KEY ("refund_id") REFERENCES "order_refunds" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_order_refund_attachments_resource_id"
        FOREIGN KEY ("resource_id") REFERENCES "resources" ("id")
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_order_refund_attachments_uploaded_by"
        FOREIGN KEY ("uploaded_by") REFERENCES "users" ("id")
        ON DELETE RESTRICT ON UPDATE CASCADE
);

-- 通用资源存储渠道设置
INSERT INTO "system_settings" ("key", "value")
VALUES ('resource_storage_channel_id', '0')
ON CONFLICT ("key") DO NOTHING;

-- 注册码状态：4 = revoked（已撤销授权）
ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2, 3, 4));
COMMENT ON COLUMN "reg_codes"."status" IS '0: unused 1: issued 2: binded 3: refunded 4: revoked';
