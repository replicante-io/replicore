#!/bin/bash
set -e

# First bootstrap the store (needed the first time).
snuba bootstrap --force
snuba migrate
