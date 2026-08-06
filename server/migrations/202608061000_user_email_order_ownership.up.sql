ALTER TABLE "users"
    ADD COLUMN "email" VARCHAR(320),
    ADD COLUMN "email_verified_at" TIMESTAMPTZ;

CREATE UNIQUE INDEX "uq_users_email_lower"
    ON "users" (LOWER("email"))
    WHERE "email" IS NOT NULL;

ALTER TABLE "orders"
    ADD COLUMN "buyer_user_id" INTEGER,
    ADD COLUMN "buyer_email" VARCHAR(320),
    ADD CONSTRAINT "fk_orders_buyer_user_id"
        FOREIGN KEY ("buyer_user_id") REFERENCES "users" ("id")
        ON DELETE SET NULL ON UPDATE CASCADE;

CREATE INDEX "idx_orders_buyer_user_created_at"
    ON "orders" ("buyer_user_id", "created_at" DESC);

CREATE INDEX "idx_orders_unowned_buyer_email"
    ON "orders" (LOWER("buyer_email"))
    WHERE "buyer_user_id" IS NULL AND "buyer_email" IS NOT NULL;

CREATE TABLE "email_verification_challenges" (
    "id" UUID PRIMARY KEY,
    "email" VARCHAR(320) NOT NULL,
    "purpose" VARCHAR(32) NOT NULL,
    "code_hash" VARCHAR(64) NOT NULL,
    "attempts" INTEGER NOT NULL DEFAULT 0,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "resend_after" TIMESTAMPTZ NOT NULL,
    "sent_at" TIMESTAMPTZ,
    "send_failed_at" TIMESTAMPTZ,
    "verified_at" TIMESTAMPTZ,
    "consumed_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "chk_email_verification_challenges_purpose"
        CHECK ("purpose" IN ('register')),
    CONSTRAINT "chk_email_verification_challenges_attempts"
        CHECK ("attempts" >= 0)
);

CREATE INDEX "idx_email_verification_challenges_email_created"
    ON "email_verification_challenges" ("email", "purpose", "created_at" DESC);

CREATE TABLE "email_verification_tokens" (
    "token_hash" VARCHAR(64) PRIMARY KEY,
    "challenge_id" UUID NOT NULL,
    "email" VARCHAR(320) NOT NULL,
    "purpose" VARCHAR(32) NOT NULL,
    "expires_at" TIMESTAMPTZ NOT NULL,
    "consumed_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_email_verification_tokens_challenge_id"
        FOREIGN KEY ("challenge_id") REFERENCES "email_verification_challenges" ("id")
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "chk_email_verification_tokens_purpose"
        CHECK ("purpose" IN ('register'))
);

CREATE INDEX "idx_email_verification_tokens_email_created"
    ON "email_verification_tokens" ("email", "purpose", "created_at" DESC);

INSERT INTO "system_settings" ("key", "value") VALUES
    ('registration_enabled', 'false'),
    ('email_service_mode', 'log'),
    ('email_from', 'LicenseHub <no-reply@example.com>'),
    ('email_smtp_host', ''),
    ('email_smtp_port', '587'),
    ('email_smtp_username', ''),
    ('email_smtp_password', ''),
    ('email_smtp_tls_mode', 'starttls')
ON CONFLICT ("key") DO NOTHING;
