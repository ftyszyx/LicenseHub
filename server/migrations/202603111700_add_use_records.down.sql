DROP TABLE IF EXISTS "use_records" CASCADE;

DELETE FROM "permissions" WHERE "id" = 20 OR "name" = 'use_records:read';
