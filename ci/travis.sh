#!/usr/bin/env bash
set -ex


# Figure out what we are asked to do and "route" to the correct script.
case "$1" in
  install)
    case "$2" in
      audit) ci/travis/audit-install.sh;;
      build) ci/travis/build-install.sh;;

      *)
        echo "Unsupported install task '$2' received"
        ;;
    esac
    ;;

  script)
    case "$2" in
      audit) ci/travis/audit-script.sh;;
      build) ci/travis/build-script.sh;;

      *)
        echo "Unsupported script task '$2' received"
        ;;
    esac
    ;;

  *)
    echo "Unsupported stage '$1' received"
    exit 1
    ;;
esac
