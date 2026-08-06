ALTER TABLE "users"
    ADD COLUMN "referrer_user_id" INTEGER,
    ADD COLUMN "referrer_bound_at" TIMESTAMPTZ,
    ADD COLUMN "registered_referral_code" VARCHAR(32),
    ADD CONSTRAINT "fk_users_referrer_user_id"
        FOREIGN KEY ("referrer_user_id") REFERENCES "users" ("id")
        ON DELETE SET NULL ON UPDATE CASCADE,
    ADD CONSTRAINT "chk_users_referrer_not_self"
        CHECK ("referrer_user_id" IS NULL OR "referrer_user_id" <> "id");

CREATE INDEX "idx_users_referrer_user_id"
    ON "users" ("referrer_user_id")
    WHERE "referrer_user_id" IS NOT NULL;

