ALTER TABLE "apps"
    ADD COLUMN IF NOT EXISTS "website_url" VARCHAR(2048);

COMMENT ON COLUMN "apps"."website_url" IS 'Application official website URL';
