DELETE FROM "role_permissions"
WHERE "role_id" = 1
  AND "permission_id" IN (
      SELECT "id" FROM "permissions"
      WHERE "name" IN (
          'storage_channels:read',
          'storage_channels:create',
          'storage_channels:update',
          'storage_channels:delete',
          'version_sync:read',
          'version_sync:create'
      )
  );

DELETE FROM "permissions"
WHERE "name" IN (
    'storage_channels:read',
    'storage_channels:create',
    'storage_channels:update',
    'storage_channels:delete',
    'version_sync:read',
    'version_sync:create'
);

DROP TABLE IF EXISTS "app_version_sync_logs";
DROP TABLE IF EXISTS "storage_channels";
