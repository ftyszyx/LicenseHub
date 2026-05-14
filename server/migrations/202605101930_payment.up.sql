CREATE TABLE "license_plans" (
    "id" SERIAL PRIMARY KEY,
    "app_id" INTEGER NOT NULL,
    "name" VARCHAR(255) NOT NULL,
    "description" TEXT,
    "price_cents" INTEGER NOT NULL,
    "code_type" SMALLINT NOT NULL DEFAULT 0,
    "valid_days" INTEGER NOT NULL DEFAULT 0,
    "total_count" INTEGER,
    "status" SMALLINT NOT NULL DEFAULT 1,
    "sort_order" INTEGER NOT NULL DEFAULT 0,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_license_plans_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "chk_license_plans_price" CHECK ("price_cents" > 0),
    CONSTRAINT "chk_license_plans_code_type" CHECK ("code_type" IN (0, 1)),
    CONSTRAINT "chk_license_plans_status" CHECK ("status" IN (0, 1))
);
CREATE INDEX idx_license_plans_app_id ON "license_plans" ("app_id");
CREATE INDEX idx_license_plans_status ON "license_plans" ("status");

CREATE TABLE "orders" (
    "id" SERIAL PRIMARY KEY,
    "order_no" VARCHAR(64) NOT NULL UNIQUE,
    "plan_id" INTEGER NOT NULL,
    "app_id" INTEGER NOT NULL,
    "amount_cents" INTEGER NOT NULL,
    "pay_type" VARCHAR(32) NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "provider" VARCHAR(32) NOT NULL DEFAULT 'wechat',
    "provider_trade_no" VARCHAR(128),
    "pay_url" TEXT,
    "qr_code" TEXT,
    "url_scheme" TEXT,
    "reg_code_id" INTEGER,
    "client_ip" VARCHAR(64),
    "provider_payload" JSONB,
    "paid_at" TIMESTAMPTZ,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_orders_plan_id" FOREIGN KEY ("plan_id") REFERENCES "license_plans" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_orders_app_id" FOREIGN KEY ("app_id") REFERENCES "apps" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_orders_reg_code_id" FOREIGN KEY ("reg_code_id") REFERENCES "reg_codes" ("id") ON DELETE SET NULL ON UPDATE CASCADE,
    CONSTRAINT "chk_orders_amount" CHECK ("amount_cents" > 0),
    CONSTRAINT "chk_orders_status" CHECK ("status" IN (0, 1, 2, 3, 4))
);
CREATE INDEX idx_orders_plan_id ON "orders" ("plan_id");
CREATE INDEX idx_orders_app_id ON "orders" ("app_id");
CREATE INDEX idx_orders_status ON "orders" ("status");
CREATE INDEX idx_orders_created_at ON "orders" ("created_at");
