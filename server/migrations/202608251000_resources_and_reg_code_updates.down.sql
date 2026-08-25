-- 已撤销授权的注册码不能回滚到不支持 revoked 状态的约束
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM "reg_codes" WHERE "status" = 4) THEN
        RAISE EXCEPTION 'cannot roll back revoked registration codes while revoked records exist';
    END IF;
END $$;

ALTER TABLE "reg_codes" DROP CONSTRAINT "chk_reg_codes_status";
ALTER TABLE "reg_codes"
    ADD CONSTRAINT "chk_reg_codes_status" CHECK ("status" IN (0, 1, 2, 3));
COMMENT ON COLUMN "reg_codes"."status" IS '0: unused 1: issued 2: binded 3: refunded';

DELETE FROM "system_settings"
WHERE "key" = 'resource_storage_channel_id';

DROP TABLE IF EXISTS "order_refund_attachments";

DELETE FROM "role_permissions"
WHERE "permission_id" IN (
    SELECT "id" FROM "permissions"
    WHERE "name" IN ('resources:read', 'resources:create', 'resources:delete')
);

DELETE FROM "permissions"
WHERE "name" IN ('resources:read', 'resources:create', 'resources:delete');

DROP TABLE IF EXISTS "resources";
