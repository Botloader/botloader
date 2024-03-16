#!/bin/sh

./build-image-base.sh

docker build -t botloader/backend -f ../cmd/backend/Dockerfile ../