#!/usr/bin/env bash
set -x
set -eo pipefail
export _PWD="$(pwd)"
export ROOT="$(git rev-parse --show-toplevel)"
source "${ROOT}/scripts/setup.sh"
cd "${ROOT}/libs/feature-flags-server" || exit
if ! command -v psql &> /dev/null
then
    >&2 echo "Error: `psql` is not installed."
    cd "${_PWD}" || exit
    exit 1
fi

if ! command -v docker &> /dev/null
then
    >&2 echo "Error: `docker` is not installed."
    cd "${_PWD}" || exit
    exit 1
fi

if ! command -v sqlx &> /dev/null
then
    >&2 echo "Error: `sqlx` is not installed."
    >&2 echo "Use the following command to install it:"
    >&2 echo "    cargo install sqlx-cli --no-default-features --features postgres"
    cd "${_PWD}" || exit
    exit 1
fi

DB_USER="${POSTGRES_USER:=postgres}"
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=feature-flag-server}"
DB_PORT="${POSTGRES_PORT:=5599}"

if [ "${SKIP_DOCKER}" != "yes" ]
then
  DOCKERS="$(docker ps -a -q --filter ancestor=postgres:15.3-alpine3.18 --format="{{.ID}}")"
  if [ -n "$DOCKERS" ]
  then
    ensure docker rm --force --volumes $DOCKERS
  fi
  ensure docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -e POSTGRES_PORT=${DB_PORT} \
    -p "${DB_PORT}":5599 \
    -d postgres:15.3-alpine3.18 \
    postgres -p ${DB_PORT} -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -p "${DB_PORT}" -U "${DB_USER}" -d postgres -c '\q'
do
    >&2 echo "Postgres is still unavailable - sleeping"
    sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"
set -x
export DATABASE_URL="postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"
ensure sqlx database create
ensure sqlx migrate run --source migrations
ensure cargo sqlx prepare
>&2 echo "Postgres has been migrated, ready to go!"
cd "${_PWD}"

cargo run -p feature-flags-server