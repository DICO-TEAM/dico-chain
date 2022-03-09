#!/usr/bin/env bash

name="dico-dev"

[[ $(killall -9 ${name}) =~ "dico-dev not lanched" ]]

echo "kill end"