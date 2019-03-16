#!/usr/bin/env bash
set -e


# Initialise MongoDB.
docker-compose exec mongo /replicore/init.sh
