#!/bin/bash
set -euo pipefail

# Deploy an instantiated platform: build service + web, ship to the VPS,
# restart the systemd unit, verify /healthz, roll back on failure.
# The web dist is swapped atomically: rsync into dist.new, then at activation
# "mv dist dist.old && mv dist.new dist" (delta 13) — the live dist path is
# never half-written.
# Rollback restores both the previous binary and the previous web dist
# (snapshotted before shipping), restarts, then re-verifies /healthz and
# reports ROLLBACK OK / ROLLBACK FAILED; the script exits non-zero either way.
# Usage: deploy.sh <platform-name>
# Env:   DEPLOY_HOST DEPLOY_USER DEPLOY_PATH DEPLOY_DOMAIN (required)
#        DEPLOY_KEY_FILE (ssh key path; default ./deploy_key)
#        DEPLOY_BINARY   (remote binary path; default /usr/local/bin/<name>-server)
# Run from the clone root (service/ and web/ subdirs).
#
# Operations note:
# - The server binary runs embedded migrations at startup, so no separate
#   `sqlx migrate run` step is required on the remote. If you prefer to run
#   migrations explicitly, have the systemd unit execute `sqlx migrate run`
#   before starting the service.
# - The systemd unit must load the required environment variables
#   (DATABASE_URL, SESSION_SIGNING_KEY, APP_URL, API_PORT, CORS_ORIGIN, etc.)
#   from a secure env file such as /etc/default/<name>.

PLATFORM="${1:?usage: deploy.sh <platform-name>}"
: "${DEPLOY_HOST:?set DEPLOY_HOST}" "${DEPLOY_USER:?set DEPLOY_USER}"
: "${DEPLOY_PATH:?set DEPLOY_PATH}" "${DEPLOY_DOMAIN:?set DEPLOY_DOMAIN}"

KEY="${DEPLOY_KEY_FILE:-deploy_key}"
BINARY="${DEPLOY_BINARY:-/usr/local/bin/${PLATFORM}-server}"
DIST="${DEPLOY_PATH}/web/dist"
SSH=(ssh -i "$KEY" -o StrictHostKeyChecking=no)
RSYNC=(rsync -az -e "ssh -i $KEY -o StrictHostKeyChecking=no")
REMOTE="${DEPLOY_USER}@${DEPLOY_HOST}"

healthz() {
  for _ in $(seq 1 10); do
    if curl -sf "https://${DEPLOY_DOMAIN}/healthz" > /dev/null 2>&1; then
      return 0
    fi
    sleep 3
  done
  return 1
}

rollback() {
  echo "Rolling back to previous release" >&2
  "${SSH[@]}" "$REMOTE" "
    restored=0
    if [ -f '${BINARY}.prev' ]; then mv '${BINARY}.prev' '$BINARY'; restored=1; fi
    if [ -d '${DIST}.prev' ]; then rm -rf '$DIST'; mv '${DIST}.prev' '$DIST'; restored=1; fi
    if [ \"\$restored\" -eq 1 ]; then
      systemctl restart '${PLATFORM}'
    else
      echo 'no previous release to restore' >&2
    fi
  " || true
  if healthz; then
    echo "ROLLBACK OK — service healthy on previous release" >&2
  else
    echo "ROLLBACK FAILED — still unhealthy after rollback; manual intervention needed" >&2
  fi
  exit 1
}

echo "=== Building $PLATFORM ==="
(cd service && cargo build --release -p server)
cp "service/target/release/server" "${PLATFORM}-server.new"

# The Nuxt generate step fetches live clinic/treatment data from the API, so we
# start a local instance of the server against the configured database.
SERVER_PID=""
start_local_server() {
  if [ -z "${DATABASE_URL:-}" ]; then
    echo "WARNING: DATABASE_URL not set; nuxt generate may fail or produce empty content" >&2
    return 0
  fi
  ./${PLATFORM}-server.new > /tmp/${PLATFORM}-build-server.log 2>&1 &
  SERVER_PID=$!
  for _ in $(seq 1 30); do
    if curl -sf http://localhost:8080/healthz > /dev/null 2>&1; then
      echo "Local build server ready (pid $SERVER_PID)" >&2
      return 0
    fi
    sleep 1
  done
  echo "ERROR: local build server failed to start" >&2
  cat /tmp/${PLATFORM}-build-server.log >&2 || true
  return 1
}
stop_local_server() {
  if [ -n "$SERVER_PID" ]; then
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
    SERVER_PID=""
  fi
}

trap stop_local_server EXIT
start_local_server
(cd web && npm ci && NUXT_PUBLIC_API_URL=http://localhost:8080 npx nuxt generate)
stop_local_server
trap - EXIT

echo "=== Shipping to $REMOTE ==="
"${SSH[@]}" "$REMOTE" "if [ -f '$BINARY' ]; then cp '$BINARY' '${BINARY}.prev'; fi"
"${RSYNC[@]}" "${PLATFORM}-server.new" "$REMOTE:${BINARY}.new"
"${SSH[@]}" "$REMOTE" "if [ -d '$DIST' ]; then rm -rf '${DIST}.prev'; cp -r '$DIST' '${DIST}.prev'; fi"
"${SSH[@]}" "$REMOTE" "rm -rf '${DIST}.new'"
"${RSYNC[@]}" --delete "web/.output/public/" "$REMOTE:${DIST}.new/"

echo "=== Activating $PLATFORM ==="
if ! "${SSH[@]}" "$REMOTE" "mv '${BINARY}.new' '$BINARY' && chmod +x '$BINARY' && { rm -rf '${DIST}.old'; if [ -d '$DIST' ]; then mv '$DIST' '${DIST}.old'; fi; mv '${DIST}.new' '$DIST'; } && systemctl restart '${PLATFORM}'"; then
  echo "Activation FAILED" >&2
  rollback
fi

echo "=== Health check ==="
if ! healthz; then
  echo "Health check FAILED" >&2
  rollback
fi

echo "=== Deployed $PLATFORM successfully ==="
