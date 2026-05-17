DELETE FROM "role_permissions"
WHERE "role_id" = 1
  AND "permission_id" IN (
      SELECT "id" FROM "permissions"
      WHERE "name" = 'dashboard:read'
  );

DELETE FROM "permissions"
WHERE "name" = 'dashboard:read';
