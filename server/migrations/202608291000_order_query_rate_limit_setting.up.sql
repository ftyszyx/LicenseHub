INSERT INTO "system_settings" ("key", "value")
VALUES ('order_query_rate_limit_per_minute', '5')
ON CONFLICT ("key") DO NOTHING;
