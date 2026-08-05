ALTER TABLE "users"
    ADD COLUMN "settlement_account" JSONB;

ALTER TABLE "distribution_commissions"
    ADD COLUMN "locked_amount_cents" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "settled_amount_cents" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "cancelled_amount_cents" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "adjustment_amount_cents" INTEGER NOT NULL DEFAULT 0;

UPDATE "distribution_commissions"
SET "cancelled_amount_cents" = "commission_amount_cents"
WHERE "status" = 4;

ALTER TABLE "distribution_commissions"
    DROP CONSTRAINT "chk_distribution_commissions_status",
    ADD CONSTRAINT "chk_distribution_commissions_status"
        CHECK ("status" IN (0, 1, 2, 3, 4, 5)),
    ADD CONSTRAINT "chk_distribution_commissions_allocated_amount"
        CHECK (
            "locked_amount_cents" >= 0
            AND "settled_amount_cents" >= 0
            AND "cancelled_amount_cents" >= 0
            AND "adjustment_amount_cents" >= 0
            AND "locked_amount_cents"
                + "settled_amount_cents"
                + "cancelled_amount_cents"
                + "adjustment_amount_cents" <= "commission_amount_cents"
        );

CREATE TABLE "distribution_settlements" (
    "id" BIGSERIAL PRIMARY KEY,
    "settlement_no" VARCHAR(64) NOT NULL UNIQUE,
    "user_id" INTEGER NOT NULL,
    "amount_cents" INTEGER NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "settlement_account" JSONB NOT NULL,
    "payment_reference" VARCHAR(255),
    "payment_proof_file_name" VARCHAR(255),
    "payment_proof_content_type" VARCHAR(128),
    "payment_proof_size" BIGINT,
    "reject_reason" TEXT,
    "requested_at" TIMESTAMPTZ NOT NULL,
    "reviewed_at" TIMESTAMPTZ,
    "paid_at" TIMESTAMPTZ,
    "reviewed_by" INTEGER,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_settlements_user_id"
        FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_settlements_reviewed_by"
        FOREIGN KEY ("reviewed_by") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_distribution_settlements_amount" CHECK ("amount_cents" > 0),
    CONSTRAINT "chk_distribution_settlements_status" CHECK ("status" IN (0, 1, 2)),
    CONSTRAINT "chk_distribution_settlements_account" CHECK (jsonb_typeof("settlement_account") = 'object'),
    CONSTRAINT "chk_distribution_settlements_paid_fields" CHECK (
        "status" <> 1 OR (
            "payment_reference" IS NOT NULL
            AND "payment_proof_file_name" IS NOT NULL
            AND "payment_proof_content_type" IS NOT NULL
            AND "payment_proof_size" IS NOT NULL
            AND "reviewed_at" IS NOT NULL
            AND "paid_at" IS NOT NULL
            AND "reviewed_by" IS NOT NULL
        )
    ),
    CONSTRAINT "chk_distribution_settlements_rejected_fields" CHECK (
        "status" <> 2 OR (
            "reject_reason" IS NOT NULL
            AND "reviewed_at" IS NOT NULL
            AND "reviewed_by" IS NOT NULL
        )
    )
);

CREATE INDEX "idx_distribution_settlements_user_created"
    ON "distribution_settlements" ("user_id", "created_at" DESC);
CREATE INDEX "idx_distribution_settlements_status_created"
    ON "distribution_settlements" ("status", "created_at" DESC);

CREATE UNIQUE INDEX "uq_distribution_settlements_user_pending"
    ON "distribution_settlements" ("user_id")
    WHERE "status" = 0;

CREATE TABLE "distribution_settlement_items" (
    "id" BIGSERIAL PRIMARY KEY,
    "settlement_id" BIGINT NOT NULL,
    "commission_id" BIGINT NOT NULL,
    "amount_cents" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_settlement_items_settlement_id"
        FOREIGN KEY ("settlement_id") REFERENCES "distribution_settlements" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_settlement_items_commission_id"
        FOREIGN KEY ("commission_id") REFERENCES "distribution_commissions" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "uq_distribution_settlement_items_pair" UNIQUE ("settlement_id", "commission_id"),
    CONSTRAINT "chk_distribution_settlement_items_amount" CHECK ("amount_cents" > 0)
);

CREATE INDEX "idx_distribution_settlement_items_commission_id"
    ON "distribution_settlement_items" ("commission_id");

CREATE TABLE "distribution_settlement_proofs" (
    "settlement_id" BIGINT PRIMARY KEY,
    "content" BYTEA NOT NULL,
    "uploaded_by" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_settlement_proofs_settlement_id"
        FOREIGN KEY ("settlement_id") REFERENCES "distribution_settlements" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_settlement_proofs_uploaded_by"
        FOREIGN KEY ("uploaded_by") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_distribution_settlement_proofs_content" CHECK (octet_length("content") > 0)
);

CREATE TABLE "distribution_commission_adjustments" (
    "id" BIGSERIAL PRIMARY KEY,
    "user_id" INTEGER NOT NULL,
    "order_id" INTEGER NOT NULL,
    "original_commission_id" BIGINT NOT NULL,
    "amount_cents" INTEGER NOT NULL,
    "offset_amount_cents" INTEGER NOT NULL DEFAULT 0,
    "reason" VARCHAR(64) NOT NULL,
    "status" SMALLINT NOT NULL DEFAULT 0,
    "operator_user_id" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_commission_adjustments_user_id"
        FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_commission_adjustments_order_id"
        FOREIGN KEY ("order_id") REFERENCES "orders" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_commission_adjustments_original_commission_id"
        FOREIGN KEY ("original_commission_id") REFERENCES "distribution_commissions" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_commission_adjustments_operator_user_id"
        FOREIGN KEY ("operator_user_id") REFERENCES "users" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "uq_distribution_commission_adjustments_refund"
        UNIQUE ("original_commission_id", "reason"),
    CONSTRAINT "chk_distribution_commission_adjustments_amount" CHECK ("amount_cents" < 0),
    CONSTRAINT "chk_distribution_commission_adjustments_offset" CHECK (
        "offset_amount_cents" >= 0 AND "offset_amount_cents" <= -"amount_cents"
    ),
    CONSTRAINT "chk_distribution_commission_adjustments_status" CHECK ("status" IN (0, 1, 2, 3))
);

CREATE INDEX "idx_distribution_commission_adjustments_user_status"
    ON "distribution_commission_adjustments" ("user_id", "status", "created_at");

CREATE TABLE "distribution_commission_adjustment_offsets" (
    "id" BIGSERIAL PRIMARY KEY,
    "adjustment_id" BIGINT NOT NULL,
    "commission_id" BIGINT NOT NULL,
    "amount_cents" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT "fk_distribution_commission_adjustment_offsets_adjustment_id"
        FOREIGN KEY ("adjustment_id") REFERENCES "distribution_commission_adjustments" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT "fk_distribution_commission_adjustment_offsets_commission_id"
        FOREIGN KEY ("commission_id") REFERENCES "distribution_commissions" ("id") ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT "chk_distribution_commission_adjustment_offsets_amount" CHECK ("amount_cents" > 0)
);

CREATE INDEX "idx_distribution_commission_adjustment_offsets_commission_id"
    ON "distribution_commission_adjustment_offsets" ("commission_id");

INSERT INTO "permissions" ("name", "resource", "action", "description") VALUES
    ('distribution:update', 'distribution', 'UPDATE', 'Distribution settlements - review and payment')
ON CONFLICT ("name") DO UPDATE SET
    "resource" = EXCLUDED."resource",
    "action" = EXCLUDED."action",
    "description" = EXCLUDED."description",
    "updated_at" = CURRENT_TIMESTAMP;

INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT 1, p."id"
FROM "permissions" p
WHERE p."name" = 'distribution:update'
  AND NOT EXISTS (
      SELECT 1
      FROM "role_permissions" rp
      WHERE rp."role_id" = 1 AND rp."permission_id" = p."id"
  );
