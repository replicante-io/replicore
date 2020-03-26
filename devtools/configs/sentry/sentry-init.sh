#!/bin/bash
set -e

# Initialise or upgrade sentry.
sentry upgrade --noinput

# Create the user (--force-update to not fail if the user exists).
sentry createuser --email 'dev@local.com' --password 'dev' --superuser --no-input || true
