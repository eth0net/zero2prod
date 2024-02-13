#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "You can install it using:"
    echo >&2 "    cargo install --version='~0.7' sqlx-cli --no-default-features --features rustls,postgres"
    exit 1
fi

CONTAINER_NAME="zero2prod_db"

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]; then
    if docker ps -a --format '{{.Names}}' | grep -q "$CONTAINER_NAME"; then
        echo >&2 "Database container $CONTAINER_NAME is already running"
    else
        echo >&2 "Starting database container $CONTAINER_NAME"
        docker run -d \
            -e POSTGRES_USER=${DB_USER} \
            -e POSTGRES_PASSWORD=${DB_PASSWORD} \
            -e POSTGRES_DB=${DB_NAME} \
            -p "${DB_PORT}":5432 \
            --name "${CONTAINER_NAME}" \
            postgres \
            postgres -N 1000
    fi
fi

if ! [ -x "$(command -v psql)" ]; then
    until docker exec -it zero2prod_db psql -h ${DB_HOST} -U ${DB_USER} -p ${DB_PORT} -d postgres -c '\q'; do
        >&2 echo "Postgres is still unavailable - sleeping"
        sleep 1
    done
else
    until PGPASSWORD="${DB_PASSWORD}" psql -h ${DB_HOST} -U ${DB_USER} -p ${DB_PORT} -d postgres -c '\q'; do
        >&2 echo "Postgres is still unavailable - sleeping"
        sleep 1
    done
fi

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}"

sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
