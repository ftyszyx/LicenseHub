DELETE FROM "role_permissions"
WHERE "permission_id" IN (
    SELECT "id" FROM "permissions" WHERE "name" = 'system_logs:read'
);

DELETE FROM "permissions" WHERE "name" = 'system_logs:read';
