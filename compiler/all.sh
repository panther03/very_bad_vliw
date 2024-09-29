#!/bin/bash
cd /home/root/cs470
. "$HOME/.cargo/env"
./build.sh
./runall.sh
./testall.sh | tee testall.log

if grep -q false testall.log; then
  exit 1
fi