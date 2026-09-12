-- An application may opt into a subset of the globally configured payment channels.
-- No rows means the application keeps the legacy global-channel behavior.
CREATE TABLE "app_payment_channels" (
    "id" SERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "payment_channel_id" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_app_payment_channels_app_id"
        FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_app_payment_channels_payment_channel_id"
        FOREIGN KEY ("payment_channel_id") REFERENCES "payment_channels" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "uq_app_payment_channels_app_channel" UNIQUE ("app_id", "payment_channel_id")
);

CREATE INDEX "idx_app_payment_channels_app_id"
    ON "app_payment_channels" ("app_id");

CREATE INDEX "idx_app_payment_channels_channel_id"
    ON "app_payment_channels" ("payment_channel_id");
