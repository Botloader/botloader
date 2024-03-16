#!/bin/sh

echo "Building backend"
./build-image-backend.sh

echo "Building frontend"
./build-image-frontend.sh