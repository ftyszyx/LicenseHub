INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, 20
WHERE EXISTS (
    SELECT 1 FROM "permissions" WHERE "id" = 20 AND "name" = 'use_records:read'
)
AND NOT EXISTS (
    SELECT 1 FROM "role_permissions" WHERE "role_id" = 1 AND "permission_id" = 20
);
