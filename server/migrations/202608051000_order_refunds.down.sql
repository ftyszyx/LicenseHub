DELETE FROM "role_permissions"
WHERE "permission_id" IN (
    SELECT "id" FROM "permissions" WHERE "name" IN ('orders:read', 'orders:update')
);

DELETE FROM "permissions" WHERE "name" IN ('orders:read', 'orders:update');

DROP TABLE IF EXISTS "order_refunds";

UPDATE "orders" SET "status" = 2 WHERE "status" = 5;

ALTER TABLE "orders"
    DROP CONSTRAINT "chk_orders_status",
    ADD CONSTRAINT "chk_orders_status" CHECK ("status" IN (0, 1, 2, 3, 4));
