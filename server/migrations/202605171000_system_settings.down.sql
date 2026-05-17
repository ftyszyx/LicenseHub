DELETE FROM "role_permissions"
WHERE "role_id" = 1
  AND "permission_id" IN (
      SELECT "id" FROM "permissions"
      WHERE "name" IN (
          'system_settings:read',
          'system_settings:update'
      )
  );

DELETE FROM "permissions"
WHERE "name" IN (
    'system_settings:read',
    'system_settings:update'
);

DROP TABLE IF EXISTS "system_settings";
