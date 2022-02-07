#!/usr/bin/env bash
set -e

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}


# https://github.com/paritytech/srtool
RUSTC_VERSION=1.57.0
PACKAGE=${PACKAGE:-tico-runtime}
BUILD_OPTS=$BUILD_OPTS

docker run --rm -it \
  -e PACKAGE=$PACKAGE \
  -e BUILD_OPTS="$BUILD_OPTS" \
  -v $PWD:/build \
  -v $TMPDIR/cargo:/cargo-home \
  --network=host \
  paritytech/srtool:$RUSTC_VERSION