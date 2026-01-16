#!/bin/sh
set -e
cmd="$@"
if [ -z "$cmd" ]; then cmd="./app_server"; fi
DATABASE_URL=postgres://${DATABASE_USER}:${DATABASE_PASSWORD}@${DATABASE_HOST}:${DATABASE_PORT}/${DATABASE_NAME}
echo "DATABASE_URL: $DATABASE_URL"
for i in $(seq 1 30); do if sqlx migrate run --database-url $DATABASE_URL ; then break; else echo "Waiting for database ($i/30)..."; sleep 2; fi; done
exec $cmd

