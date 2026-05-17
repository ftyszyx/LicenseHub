CREATE TABLE "system_settings" (
    "key" VARCHAR(128) PRIMARY KEY,
    "value" TEXT NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "chk_system_settings_key" CHECK (length(trim("key")) > 0)
);

INSERT INTO "system_settings" ("key", "value")
VALUES ('storefront_title', 'LicenseHub')
ON CONFLICT ("key") DO UPDATE SET
    "value" = EXCLUDED."value",
    "updated_at" = CURRENT_TIMESTAMP;

SELECT setval(
    pg_get_serial_sequence('permissions', 'id'),
    COALESCE((SELECT MAX("id") FROM "permissions"), 1),
    true
);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('system_settings:read', 'system_settings', 'READ', 'System settings - read'),
    ('system_settings:update', 'system_settings', 'UPDATE', 'System settings - update')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p.name IN (
    'system_settings:read',
    'system_settings:update'
)
AND NOT EXISTS (
    SELECT 1
    FROM "role_permissions" rp
    WHERE rp.role_id = 1 AND rp.permission_id = p.id
);
