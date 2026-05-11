CREATE TABLE "order_events" (
    "id" BIGSERIAL PRIMARY KEY,
    "order_id" INTEGER NOT NULL,
    "order_no" VARCHAR(64) NOT NULL,
    "event_type" VARCHAR(64) NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "payload" JSONB,
    "processed_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_order_events_order_id" FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "chk_order_events_status" CHECK ("status" IN (0, 1, 2))
);
CREATE INDEX idx_order_events_status_id ON "order_events" ("status", "id");
CREATE INDEX idx_order_events_order_id ON "order_events" ("order_id");
