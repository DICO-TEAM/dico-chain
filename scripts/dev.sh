#!/usr/bin/env bash
set -e

export BUILD_ID=dontKillMe

echo "***  chain service start***"

nohup dico --dev --ws-external >> chain.log 2>&1 &

sleep 1

echo "***  chain service end***"
