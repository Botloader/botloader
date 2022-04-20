#!/bin/sh

echo "Building base"
./build-image-base.sh

echo "Building broker"
./build-image-broker.sh

echo "Building schedulerwithworker"
./build-image-schedulerwithworker.sh

echo "Building webapi"
./build-image-webapi.sh

echo "Building frontend"
./build-image-frontend.sh