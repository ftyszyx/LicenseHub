DELETE FROM "role_permissions"
WHERE "role_id" = 1
  AND "permission_id" IN (
      SELECT "id" FROM "permissions"
      WHERE "name" IN (
          'payment_settings:read',
          'payment_settings:create',
          'payment_settings:update',
          'payment_settings:delete'
      )
  );

DELETE FROM "permissions"
WHERE "name" IN (
    'payment_settings:read',
    'payment_settings:create',
    'payment_settings:update',
    'payment_settings:delete'
);

DROP TABLE IF EXISTS "payment_channels";

ALTER TABLE "orders" ALTER COLUMN "pay_type" TYPE VARCHAR(32);
