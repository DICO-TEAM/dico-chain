#!/usr/bin/env bash

name="dico"

[[ $(killall -9 ${name}) =~ "dico not lanched" ]]

echo "kill end"