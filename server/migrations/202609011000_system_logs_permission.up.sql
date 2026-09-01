SELECT setval(
    pg_get_serial_sequence('permissions', 'id'),
    COALESCE((SELECT MAX("id") FROM "permissions"), 1),
    true
);

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('system_logs:read', 'system_logs', 'READ', 'System logs - read')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p.id
FROM "permissions" p
WHERE p.name = 'system_logs:read'
AND NOT EXISTS (
    SELECT 1
    FROM "role_permissions" rp
    WHERE rp.role_id = 1 AND rp.permission_id = p.id
);
