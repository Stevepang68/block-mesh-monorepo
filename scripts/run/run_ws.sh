#!/usr/bin/env bash
set -x
export ENFORCE_KEYPAIR="true"
export APP_ENVIRONMENT="local"
export SENTRY_LAYER="false"
export SENTRY_SAMPLE_RATE="1.0"
export _PWD="$(pwd)"
export ROOT="$(git rev-parse --show-toplevel)"
source "${ROOT}/scripts/setup.sh"
set +x
export AGG_SIZE=1
export ENFORCE_VERSION="true"
export CHANNEL_AGG_SIZE=1
export ADMIN_PARAM="test"
source "${ROOT}/scripts/setup.sh"
export DATABASE_URL="postgres://postgres:password@localhost:5559/block-mesh"
export WRITE_DATABASE_URL="postgres://postgres:password@localhost:5559/block-mesh"
export FOLLOWER_DATABASE_URL="postgres://postgres:password@localhost:5559/block-mesh"
export CHANNEL_DATABASE_URL="postgres://postgres:password@localhost:5559/block-mesh"
export REDIS_URL="redis://127.0.0.1:6379"
if [ -f "${ROOT}/.env" ] ; then
  source "${ROOT}/.env"
fi
#ensure "${ROOT}/scripts/init_db.sh"
#cargo run -p block-mesh-manager-ws | bunyan
#export RUSTFLAGS="--cfg tokio_unstable" ; export CARGO_TARGET_DIR="${ROOT}/target/tokio-console"
cargo watch --watch libs --shell "cargo run -p block-mesh-manager-ws"
