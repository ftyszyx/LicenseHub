ALTER TABLE "apps"
    ADD COLUMN "manifest_extra" JSONB NOT NULL DEFAULT '{}'::jsonb;
