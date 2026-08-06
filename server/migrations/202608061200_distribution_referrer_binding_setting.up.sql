INSERT INTO "system_settings" ("key", "value")
VALUES ('distribution_referrer_binding_enabled', 'false')
ON CONFLICT ("key") DO NOTHING;

