#!/usr/bin/env bash
set -e
# usage: NEW_VERSION=v0.9.28 scripts/bump.sh
# The following line ensure we run from the project root
PROJECT_ROOT=`git rev-parse --show-toplevel`
cd ${PROJECT_ROOT}


OLD_VERSION=`grep "^substrate-build-script-utils" ./node/dico/Cargo.toml | egrep -o "(v[0-9\.]+)"`
NEW_VERSION=${NEW_VERSION:-v0.9.18}

cargo_toml_list=$(find . -name "Cargo.toml")


echo "Bump substrate from polkadot-${OLD_VERSION} to polkadot-${NEW_VERSION}"
for cargo_toml in $cargo_toml_list
do
    sed -i "s/$OLD_VERSION/$NEW_VERSION/g" $cargo_toml
done