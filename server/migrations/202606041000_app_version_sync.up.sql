CREATE TABLE "storage_channels" (
    "id" SERIAL PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 1,
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "config" JSONB NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "chk_storage_channels_provider" CHECK ("provider" IN ('aliyun_oss', 'cloudflare_r2')),
    CONSTRAINT "chk_storage_channels_status" CHECK ("status" IN (0, 1)),
    CONSTRAINT "chk_storage_channels_name" CHECK (length(trim("name")) > 0)
);

CREATE INDEX idx_storage_channels_provider ON "storage_channels" ("provider");
CREATE INDEX idx_storage_channels_status_sort ON "storage_channels" ("status", "sort_order", "id");

CREATE TABLE "app_version_sync_logs" (
    "id" BIGSERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "storage_channel_id" INTEGER NOT NULL,
    "provider" VARCHAR(32) NOT NULL,
    "object_key" VARCHAR(512) NOT NULL,
    "public_url" TEXT NOT NULL,
    "manifest" JSONB NOT NULL,
    "status" SMALLINT NOT NULL,
    "error_message" TEXT,
    "etag" VARCHAR(255),
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "finished_at" TIMESTAMPTZ,
    CONSTRAINT "fk_app_version_sync_logs_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_app_version_sync_logs_storage_channel_id" FOREIGN KEY ("storage_channel_id") REFERENCES "storage_channels" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_app_version_sync_logs_provider" CHECK ("provider" IN ('aliyun_oss', 'cloudflare_r2')),
    CONSTRAINT "chk_app_version_sync_logs_status" CHECK ("status" IN (0, 1, 2)),
    CONSTRAINT "chk_app_version_sync_logs_object_key" CHECK (length(trim("object_key")) > 0)
);

CREATE INDEX idx_app_version_sync_logs_app_created ON "app_version_sync_logs" ("app_id", "created_at" DESC);
CREATE INDEX idx_app_version_sync_logs_channel_created ON "app_version_sync_logs" ("storage_channel_id", "created_at" DESC);
CREATE INDEX idx_app_version_sync_logs_status_created ON "app_version_sync_logs" ("status", "created_at" DESC);

SELECT setval(
    pg_get_serial_sequence('permissions', 'id'),
    COALESCE((SELECT MAX("id") FROM "permissions"), 1),
    true
);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('storage_channels:read', 'storage_channels', 'READ', 'Storage channels - read'),
    ('storage_channels:create', 'storage_channels', 'CREATE', 'Storage channels - create'),
    ('storage_channels:update', 'storage_channels', 'UPDATE', 'Storage channels - update'),
    ('storage_channels:delete', 'storage_channels', 'DELETE', 'Storage channels - delete'),
    ('version_sync:read', 'version_sync', 'READ', 'App version sync logs - read'),
    ('version_sync:create', 'version_sync', 'CREATE', 'App version sync - create')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p.name IN (
    'storage_channels:read',
    'storage_channels:create',
    'storage_channels:update',
    'storage_channels:delete',
    'version_sync:read',
    'version_sync:create'
)
AND NOT EXISTS (
    SELECT 1
    FROM "role_permissions" rp
    WHERE rp.role_id = 1 AND rp.permission_id = p.id
);
