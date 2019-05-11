#!/usr/bin/env bash
set -e


# Initialise MongoDB.
docker-compose exec mongo /replicore/init.sh


# Sentry is optional (and also takes forever so do it last).
if docker-compose ps sentry-web > /dev/null 2>&1; then
  echo "==> Sentry DB init/sync ..."
  docker-compose exec sentry-web sentry upgrade
fi
