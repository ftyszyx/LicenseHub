import argparse
import os
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any, Dict, List, Optional, Sequence

import psycopg2
import psycopg2.extras
from psycopg2 import sql


def _utcnow() -> datetime:
    return datetime.now(timezone.utc)


@dataclass(frozen=True)
class Dsns:
    source: str
    target: str


def _connect(dsn: str):
    return psycopg2.connect(dsn)


def _fetch_all(cur, query: str) -> List[Dict[str, Any]]:
    cur.execute(query)
    rows = cur.fetchall()
    return [dict(r) for r in rows]


def _truncate_target(cur) -> None:
    cur.execute('TRUNCATE TABLE "reg_codes" RESTART IDENTITY CASCADE;')
    cur.execute('TRUNCATE TABLE "app_devices" RESTART IDENTITY CASCADE;')
    cur.execute('TRUNCATE TABLE "apps" RESTART IDENTITY CASCADE;')


def _set_sequence(cur, table: str, column: str = "id") -> None:
    cur.execute(
        sql.SQL("SELECT COALESCE(MAX({col}), 0) FROM {tbl}").format(
            col=sql.Identifier(column),
            tbl=sql.Identifier(table),
        )
    )
    max_id = cur.fetchone()[0]
    cur.execute(
        sql.SQL("SELECT setval(pg_get_serial_sequence({tbl}, {col}), %s, %s)").format(
            tbl=sql.Literal(table),
            col=sql.Literal(column),
        ),
        (max_id, bool(max_id)),
    )


def _insert_apps(cur, apps: Sequence[Dict[str, Any]]) -> int:
    if not apps:
        return 0
    cols = (
        "id",
        "name",
        "app_id",
        "app_vername",
        "app_vercode",
        "app_download_url",
        "app_res_url",
        "app_update_info",
        "code_type",
        "app_valid_key",
        "trial_days",
        "trial_num",
        "sort_order",
        "status",
        "created_at",
        "updated_at",
    )
    values = []
    for a in apps:
        values.append(
            (
                a["id"],
                a["name"],
                a["app_id"],
                a["app_vername"],
                a["app_vercode"],
                a["app_download_url"],
                a["app_res_url"],
                a.get("app_update_info"),
                0,
                a.get("app_valid_key", ""),
                a.get("trial_days", 0),
                0,
                a.get("sort_order", 0),
                a.get("status", 0),
                a.get("created_at") or _utcnow(),
                a.get("updated_at") or _utcnow(),
            )
        )
    psycopg2.extras.execute_values(
        cur,
        f'INSERT INTO "apps" ({", ".join(cols)}) VALUES %s',
        values,
        page_size=1000,
    )
    return len(values)


def _insert_app_devices(cur, devices: Sequence[Dict[str, Any]]) -> int:
    if not devices:
        return 0
    cols = (
        "id",
        "app_id",
        "device_id",
        "device_info",
        "expire_time",
        "remaining",
        "created_at",
        "updated_at",
    )
    values = []
    for d in devices:
        created_at = d.get("bind_time") or _utcnow()
        values.append(
            (
                d["id"],
                d["app_id"],
                d["device_id"],
                d.get("device_info"),
                d.get("expire_time"),
                None,
                created_at,
                created_at,
            )
        )
    psycopg2.extras.execute_values(
        cur,
        f'INSERT INTO "app_devices" ({", ".join(cols)}) VALUES %s',
        values,
        page_size=1000,
    )
    return len(values)


def _map_reg_code_status(src_status: int, device_id: Optional[int]) -> int:
    if src_status == 0:
        return 0
    if src_status == 1:
        return 2
    if src_status == 2:
        return 1
    if device_id is not None:
        return 2
    return 1


def _insert_reg_codes(cur, codes: Sequence[Dict[str, Any]], skip_expired: bool) -> int:
    if not codes:
        return 0
    cols = (
        "id",
        "code",
        "app_id",
        "valid_days",
        "max_devices",
        "status",
        "binding_time",
        "code_type",
        "total_count",
        "device_id",
        "created_at",
        "updated_at",
    )
    values = []
    for c in codes:
        if skip_expired and int(c.get("status", 0)) == 2:
            continue
        device_id = c.get("device_id")
        created_at = c.get("created_at") or _utcnow()
        values.append(
            (
                c["id"],
                c["code"],
                c["app_id"],
                c.get("valid_days", 0),
                c.get("max_devices", 1),
                _map_reg_code_status(int(c.get("status", 0)), device_id),
                c.get("binding_time"),
                int(c.get("code_type", 0)),
                c.get("total_count"),
                device_id,
                created_at,
                c.get("updated_at") or created_at,
            )
        )
    if not values:
        return 0
    psycopg2.extras.execute_values(
        cur,
        f'INSERT INTO "reg_codes" ({", ".join(cols)}) VALUES %s',
        values,
        page_size=1000,
    )
    return len(values)


def main(argv: Optional[Sequence[str]] = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-dsn", default=os.getenv("SOURCE_DSN"))
    parser.add_argument("--target-dsn", default=os.getenv("TARGET_DSN"))
    parser.add_argument("--truncate-target", action="store_true")
    parser.add_argument("--skip-expired", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)

    if not args.source_dsn or not args.target_dsn:
        raise SystemExit("SOURCE_DSN/TARGET_DSN (or --source-dsn/--target-dsn) is required")

    dsns = Dsns(source=args.source_dsn, target=args.target_dsn)

    with _connect(dsns.source) as src_conn, _connect(dsns.target) as dst_conn:
        src_conn.autocommit = True
        dst_conn.autocommit = False

        with src_conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as src_cur, dst_conn.cursor() as dst_cur:
            apps = _fetch_all(src_cur, 'SELECT * FROM "apps" ORDER BY id ASC')
            devices = _fetch_all(src_cur, 'SELECT * FROM "app_devices" ORDER BY id ASC')
            codes = _fetch_all(src_cur, 'SELECT * FROM "reg_codes" ORDER BY id ASC')

            if args.dry_run:
                print(f"apps: {len(apps)}")
                print(f"app_devices: {len(devices)}")
                print(f"reg_codes: {len(codes)}")
                return 0

            try:
                if args.truncate_target:
                    _truncate_target(dst_cur)

                inserted_apps = _insert_apps(dst_cur, apps)
                inserted_devices = _insert_app_devices(dst_cur, devices)
                inserted_codes = _insert_reg_codes(dst_cur, codes, skip_expired=args.skip_expired)

                _set_sequence(dst_cur, "apps")
                _set_sequence(dst_cur, "app_devices")
                _set_sequence(dst_cur, "reg_codes")

                dst_conn.commit()
                print(f"inserted apps: {inserted_apps}")
                print(f"inserted app_devices: {inserted_devices}")
                print(f"inserted reg_codes: {inserted_codes}")
            except Exception:
                dst_conn.rollback()
                raise

    return 0


if __name__ == "__main__":
    sys.exit(main())
