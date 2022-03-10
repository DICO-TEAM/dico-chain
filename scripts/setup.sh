#!/usr/bin/env bash

set -e

install_depenencies() {
  echo "------------Set up rust development environment--------------"
  curl https://sh.rustup.rs -sSf | sh
  if [ $? -ne 0 ]; then
    echo "set rustup development  failed"
    exit 1
  fi
}

install_depenencies
