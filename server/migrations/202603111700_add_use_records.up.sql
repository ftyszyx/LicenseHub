INSERT INTO "permissions" ("id", "name", "resource", "action", "description") VALUES (20, 'use_records:read', 'use_records', 'READ', '使用记录-读取');

CREATE TABLE "use_records" (
    "id" SERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "device_id" VARCHAR NOT NULL,
    "use_count" INTEGER NOT NULL DEFAULT 1,
    "use_info" JSONB,
    "time" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_use_records_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "chk_use_records_use_count" CHECK ("use_count" > 0)
);

CREATE INDEX idx_use_records_app_id ON "use_records" ("app_id");
CREATE INDEX idx_use_records_device_id ON "use_records" ("device_id");
CREATE INDEX idx_use_records_time ON "use_records" ("time");
