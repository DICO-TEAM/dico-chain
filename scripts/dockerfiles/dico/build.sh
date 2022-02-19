#!/usr/bin/env bash
# Forked from: https://github.com/paritytech/polkadot
set -e

pushd .

# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd $PROJECT_ROOT


# Find the current version from Cargo.toml
VERSION=`grep "^version" ./node/dico/Cargo.toml | egrep -o "([0-9\.]+)"`
GITUSER=dicoteam
GITREPO=dico

# Build the image
echo "Building ${GITUSER}/${GITREPO}:latest docker image, hang on!"
time docker build --network=host -f ./scripts/dockerfiles/dico/dico_builder.Dockerfile -t ${GITUSER}/${GITREPO}:latest .
docker tag ${GITUSER}/${GITREPO}:latest ${GITUSER}/${GITREPO}:v${VERSION}

# Show the list of available images for this repo
echo "Image is ready"
docker images | grep ${GITREPO}

popd