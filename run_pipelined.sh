#!/bin/bash
./test.sh $1
cd build
./SingleCoreTest
cd ..
