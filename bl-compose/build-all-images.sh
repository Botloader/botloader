#!/bin/sh

echo "Building base"
./build-image-base.sh

echo "Building broker"
./build-image-broker.sh

echo "Building schedulerwithworker"
./build-image-schedulerwithworker.sh